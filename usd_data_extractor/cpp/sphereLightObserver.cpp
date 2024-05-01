#include "sphereLightObserver.h"
#include "usd_data_extractor/src/bridge.rs.h"

SphereLightObserver::SphereLightObserver() {}

SphereLightObserver::~SphereLightObserver() {}

void
SphereLightObserver::PrimsAdded(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::AddedPrimEntries& entries)
{
  for (const auto entry : entries) {
    auto primType = entry.primType;

    if (primType != TypeToken) {
      continue;
    }

    // stageに追加されたSphereLightを記録する
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
      // _addedされたSphereLightとしてdiffに登録する
      _added.emplace(entry.primPath);
    }
  }
}

void
SphereLightObserver::PrimsRemoved(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::RemovedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // _lightPathに記録されていない場合は無視する
    if (_lightPaths.find(entry.primPath) == _lightPaths.end()) {
      continue;
    }

    // stageから削除されたSphereLightを記録から削除する
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
      // _removedされたSphereLightとしてdiffに登録する
      _removed.emplace(entry.primPath);
    }
  }
}

void
SphereLightObserver::PrimsDirtied(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::DirtiedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // _lightPathに記録されていない場合は無視する
    if (_lightPaths.find(entry.primPath) == _lightPaths.end()) {
      continue;
    }

    // このフレーム中でaddedな場合は、addedですべての情報を送るので追加で差分を送る必要はない
    // そのため、addedされたSphereLightの場合はdirtiedを無視する
    if (_added.find(entry.primPath) != _added.end()) {
      continue;
    }

    // dirtiedされたらdiffに記録する
    _dirtied.emplace(entry.primPath);
  }
}

void
SphereLightObserver::PrimsRenamed(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::RenamedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // SphereLightPathに記録されていない場合は無視する
    if (_lightPaths.find(entry.oldPrimPath) == _lightPaths.end()) {
      continue;
    }

    // stageからrenameされたSphereLightを記録から削除し、新しい名前で記録する
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
        // _removedされたSphereLightとしてdiffに登録する
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
        // _addedされたSphereLightとしてdiffに登録する
        _added.emplace(entry.newPrimPath);
      }
    }
  }
}

void
SphereLightObserver::ClearDiff()
{
  // 各種diffの記録をクリアする
  _added.clear();
  _removed.clear();
  _dirtied.clear();
}

void
UpdateDiff(const HdSceneIndexBase& sceneIndex,
           UsdDataDiff& diff,
           const SdfPath path)
{
  auto pathString = rust::String(path.GetText());

  diff.add_or_update_sphere_light(pathString);

  auto transformMatrixSource =
    sceneIndex.GetDataSource(path, SphereLightObserver::TransformMatrixLocator);
  if (transformMatrixSource) {
    auto sampledTransformMatrixSource =
      HdSampledDataSource::Cast(transformMatrixSource);
    auto value = sampledTransformMatrixSource->GetValue(0);
    auto matrix = value.Get<GfMatrix4d>();
    auto matrixArray = matrix.GetArray();
    std::array<float, 16> matrixData;
    for (int i = 0; i < 16; i++) {
      matrixData[i] = matrixArray[i];
    }
    auto data = rust::Slice<const float>(matrixData.data(), 16);
    diff.add_or_update_sphere_light_transform_matrix(pathString, data);
  }

  auto materialTerminalSource = sceneIndex.GetDataSource(
    path, SphereLightObserver::MaterialTerminalLocator);
  if (materialTerminalSource) {
    auto sampledMaterialTerminalSource =
      HdSampledDataSource::Cast(materialTerminalSource);
    auto value = sampledMaterialTerminalSource->GetValue(0);
    auto terminal = value.Get<TfToken>();

    auto colorLocator =
      SphereLightObserver::MaterialNodesLocator.Append(terminal).Append(
        SphereLightObserver::ColorParameterLocator);
    auto colorSource = sceneIndex.GetDataSource(path, colorLocator);
    if (colorSource) {
      auto sampledColorSource = HdSampledDataSource::Cast(colorSource);
      auto value = sampledColorSource->GetValue(0);
      auto color = value.Get<GfVec3f>();
      diff.add_or_update_sphere_light_color(
        pathString, color[0], color[1], color[2]);
    }

    auto intensityLocator =
      SphereLightObserver::MaterialNodesLocator.Append(terminal).Append(
        SphereLightObserver::IntensityParameterLocator);
    auto intensitySource = sceneIndex.GetDataSource(path, intensityLocator);
    if (intensitySource) {
      auto sampledIntensitySource = HdSampledDataSource::Cast(intensitySource);
      auto value = sampledIntensitySource->GetValue(0);
      auto intensity = value.Get<float>();
      diff.add_or_update_sphere_light_intensity(pathString, intensity);
    }

    auto angleLocator =
      SphereLightObserver::MaterialNodesLocator.Append(terminal).Append(
        SphereLightObserver::AngleParameterLocator);
    auto angleSource = sceneIndex.GetDataSource(path, angleLocator);
    if (angleSource) {
      auto sampledAngleSource = HdSampledDataSource::Cast(angleSource);
      auto value = sampledAngleSource->GetValue(0);
      auto angle = value.Get<float>();
      diff.add_or_update_sphere_light_cone_angle(pathString, angle);
    }

    auto softnessLocator =
      SphereLightObserver::MaterialNodesLocator.Append(terminal).Append(
        SphereLightObserver::SoftnessParameterLocator);
    auto softnessSource = sceneIndex.GetDataSource(path, softnessLocator);
    if (softnessSource) {
      auto sampledSoftnessSource = HdSampledDataSource::Cast(softnessSource);
      auto value = sampledSoftnessSource->GetValue(0);
      auto softness = value.Get<float>();
      diff.add_or_update_sphere_light_cone_softness(pathString, softness);
    }
  }
}

void
SphereLightObserver::GetDiff(const HdSceneIndexBase& sceneIndex,
                             UsdDataDiff& diff)
{
  // addedされたSphereLightの情報をdiffに登録する
  for (const auto& path : _added) {
    UpdateDiff(sceneIndex, diff, path);
  }

  // removedされたSphereLightの情報をdiffに登録する
  for (const auto& path : _removed) {
    auto pathString = rust::String(path.GetText());
    diff.destroy_sphere_light(pathString);
  }

  // dirtiedされたSphereLightの情報をdiffに登録する
  for (const auto& path : _dirtied) {
    UpdateDiff(sceneIndex, diff, path);
  }
}
