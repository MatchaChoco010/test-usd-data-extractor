#include "renderSettingsObserver.h"
#include "usd_data_extractor/src/bridge.rs.h"

RenderSettingsObserver::RenderSettingsObserver() {}

RenderSettingsObserver::~RenderSettingsObserver() {}

void
RenderSettingsObserver::PrimsAdded(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::AddedPrimEntries& entries)
{
  for (const auto entry : entries) {
    auto primType = entry.primType;

    if (primType != TypeToken) {
      continue;
    }

    // stageに追加されたRenderSettingsを記録する
    _lightPaths.insert(entry.primPath);

    if (_removed.find(entry.primPath) != _removed.end()) {
      // このDiff中ですでにremovedされているDiffがある場合、
      // removedを取り消してaddedとして扱う
      _removed.erase(entry.primPath);
      _added.emplace(entry.primPath);
    } else if (_dirtied.find(entry.primPath) != _dirtied.end()) {
      // このDiff中ですでにdirtiedされているDiffがある場合、
      // dirtiedを取り消してaddedとして扱う
      _dirtied.erase(entry.primPath);
      _added.emplace(entry.primPath);
    } else {
      // _addedされたRenderSettingsとしてdiffに登録する
      _added.emplace(entry.primPath);
    }
  }
}

void
RenderSettingsObserver::PrimsRemoved(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::RemovedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // _lightPathに記録されていない場合は無視する
    if (_lightPaths.find(entry.primPath) == _lightPaths.end()) {
      continue;
    }

    // stageから削除されたRenderSettingsを記録から削除する
    _lightPaths.erase(entry.primPath);

    if (_added.find(entry.primPath) != _added.end()) {
      // このDiff中ですでにaddedされているDiffがある場合、
      // addedを取り消して差分はなかったことにする
      _added.erase(entry.primPath);
    } else if (_dirtied.find(entry.primPath) != _dirtied.end()) {
      // このDiff中ですでにdirtiedされているDiffがある場合、
      // そのdirtiedは削除されるので取り消してremovedだけを記録する
      _dirtied.erase(entry.primPath);
      _removed.emplace(entry.primPath);
    } else {
      // _removedされたRenderSettingsとしてdiffに登録する
      _removed.emplace(entry.primPath);
    }
  }
}

void
RenderSettingsObserver::PrimsDirtied(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::DirtiedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // _lightPathに記録されていない場合は無視する
    if (_lightPaths.find(entry.primPath) == _lightPaths.end()) {
      continue;
    }

    // このフレーム中でaddedな場合は、addedですべての情報を送るので追加で差分を送る必要はない
    // そのため、addedされたRenderSettingsの場合はdirtiedを無視する
    if (_added.find(entry.primPath) != _added.end()) {
      continue;
    }

    // dirtiedされたらdiffに記録する
    _dirtied.emplace(entry.primPath);
  }
}

void
RenderSettingsObserver::PrimsRenamed(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::RenamedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // RenderSettingsPathに記録されていない場合は無視する
    if (_lightPaths.find(entry.oldPrimPath) == _lightPaths.end()) {
      continue;
    }

    // stageからrenameされたRenderSettingsを記録から削除し、新しい名前で記録する
    _lightPaths.erase(entry.oldPrimPath);
    _lightPaths.insert(entry.newPrimPath);

    // oldPathをremoveする
    {
      if (_added.find(entry.oldPrimPath) != _added.end()) {
        // このDiff中ですでにaddedされているDiffがある場合、
        // addedを取り消して差分はなかったことにする
        _added.erase(entry.oldPrimPath);
      } else if (_dirtied.find(entry.oldPrimPath) != _dirtied.end()) {
        // このDiff中ですでにdirtiedされているDiffがある場合、
        // そのdirtiedは削除されるので取り消す
        _dirtied.erase(entry.oldPrimPath);
        _removed.emplace(entry.oldPrimPath);
      } else {
        // _removedされたRenderSettingsとしてdiffに登録する
        _removed.emplace(entry.oldPrimPath);
      }
    }

    // newPathをaddする
    {
      if (_removed.find(entry.newPrimPath) != _removed.end()) {
        // このDiff中ですでにremovedされているDiffがある場合、
        // removedを取り消してaddedとして扱う
        _removed.erase(entry.newPrimPath);
        _added.emplace(entry.newPrimPath);
      } else if (_dirtied.find(entry.newPrimPath) != _dirtied.end()) {
        // このDiff中ですでにdirtiedされているDiffがある場合、
        // dirtiedを取り消してaddedとして扱う
        _dirtied.erase(entry.newPrimPath);
        _added.emplace(entry.newPrimPath);
      } else {
        // _addedされたRenderSettingsとしてdiffに登録する
        _added.emplace(entry.newPrimPath);
      }
    }
  }
}

void
RenderSettingsObserver::ClearDiff()
{
  // 各種diffの記録をクリアする
  _added.clear();
  _removed.clear();
  _dirtied.clear();
}

void
RenderSettingsObserver::_UpdateDiff(const HdSceneIndexBase& sceneIndex,
                                    UsdDataDiff& diff,
                                    const SdfPath path) const
{
  auto pathString = rust::String(path.GetText());

  diff.add_or_update_render_settings(pathString);

  auto renderProductsSource =
    sceneIndex.GetDataSource(path, RenderProductsLocator);
  if (renderProductsSource) {
    auto vectorRenderProductsSource =
      HdVectorDataSource::Cast(renderProductsSource);
    for (size_t i = 0; i < vectorRenderProductsSource->GetNumElements(); i++) {
      auto renderProductSource = vectorRenderProductsSource->GetElement(i);
      auto containerRenderProductSource =
        HdContainerDataSource::Cast(renderProductSource);

      auto renderProductPathSource =
        containerRenderProductSource->Get(TfToken("path"));
      auto sampledRenderProductPath =
        HdSampledDataSource::Cast(renderProductPathSource);
      auto renderProductPathValue = sampledRenderProductPath->GetValue(0);
      auto renderProductPath = renderProductPathValue.Get<SdfPath>();
      auto renderProductPathString = rust::String(renderProductPath.GetText());

      auto cameraPrimSource =
        containerRenderProductSource->Get(TfToken("cameraPrim"));
      auto sampledCameraPrimSource =
        HdSampledDataSource::Cast(cameraPrimSource);
      auto cameraPrimValue = sampledCameraPrimSource->GetValue(0);
      auto cameraPrim = cameraPrimValue.Get<SdfPath>();
      auto cameraPrimString = rust::String(cameraPrim.GetText());

      diff.add_or_update_render_settings_render_product(
        pathString, renderProductPathString, cameraPrimString);
    }
  }
}

void
RenderSettingsObserver::GetDiff(const HdSceneIndexBase& sceneIndex,
                                UsdDataDiff& diff)
{
  // addedされたRenderSettingsの情報をdiffに登録する
  for (const auto& path : _added) {
    _UpdateDiff(sceneIndex, diff, path);
  }

  // removedされたRenderSettingsの情報をdiffに登録する
  for (const auto& path : _removed) {
    auto pathString = rust::String(path.GetText());
    diff.destroy_render_settings(pathString);
  }

  // dirtiedされたRenderSettingsの情報をdiffに登録する
  for (const auto& path : _dirtied) {
    _UpdateDiff(sceneIndex, diff, path);
  }
}
