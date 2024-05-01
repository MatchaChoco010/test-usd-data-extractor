#include "cameraObserver.h"
#include "usd_data_extractor/src/bridge.rs.h"

CameraObserver::CameraObserver() {}

CameraObserver::~CameraObserver() {}

void
CameraObserver::PrimsAdded(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::AddedPrimEntries& entries)
{
  for (const auto entry : entries) {
    auto primType = entry.primType;

    if (primType != TypeToken) {
      continue;
    }

    // stageに追加されたCameraを記録する
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
      // _addedされたCameraとしてdiffに登録する
      _added.emplace(entry.primPath);
    }
  }
}

void
CameraObserver::PrimsRemoved(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::RemovedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // _lightPathに記録されていない場合は無視する
    if (_lightPaths.find(entry.primPath) == _lightPaths.end()) {
      continue;
    }

    // stageから削除されたCameraを記録から削除する
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
      // _removedされたCameraとしてdiffに登録する
      _removed.emplace(entry.primPath);
    }
  }
}

void
CameraObserver::PrimsDirtied(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::DirtiedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // _lightPathに記録されていない場合は無視する
    if (_lightPaths.find(entry.primPath) == _lightPaths.end()) {
      continue;
    }

    // このフレーム中でaddedな場合は、addedですべての情報を送るので追加で差分を送る必要はない
    // そのため、addedされたCameraの場合はdirtiedを無視する
    if (_added.find(entry.primPath) != _added.end()) {
      continue;
    }

    // dirtiedされたらdiffに記録する
    _dirtied.emplace(entry.primPath);
  }
}

void
CameraObserver::PrimsRenamed(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::RenamedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // CameraPathに記録されていない場合は無視する
    if (_lightPaths.find(entry.oldPrimPath) == _lightPaths.end()) {
      continue;
    }

    // stageからrenameされたCameraを記録から削除し、新しい名前で記録する
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
        // _removedされたCameraとしてdiffに登録する
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
        // _addedされたCameraとしてdiffに登録する
        _added.emplace(entry.newPrimPath);
      }
    }
  }
}

void
CameraObserver::ClearDiff()
{
  // 各種diffの記録をクリアする
  _added.clear();
  _removed.clear();
  _dirtied.clear();
}

void
CameraObserver::_UpdateDiff(const HdSceneIndexBase& sceneIndex,
                            UsdDataDiff& diff,
                            const SdfPath path) const
{
  auto pathString = rust::String(path.GetText());

  diff.add_or_update_camera(pathString);

  auto transformMatrixSource =
    sceneIndex.GetDataSource(path, TransformMatrixLocator);
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
    diff.add_or_update_camera_transform_matrix(pathString, data);
  }

  auto focalLengthSource = sceneIndex.GetDataSource(path, FocalLengthLocator);
  if (focalLengthSource) {
    auto sampledFocalLengthSource =
      HdSampledDataSource::Cast(focalLengthSource);
    auto value = sampledFocalLengthSource->GetValue(0);
    auto focalLength = value.Get<float>();
    diff.add_or_update_camera_focal_length(pathString, focalLength);
  }

  auto verticalApertureSource =
    sceneIndex.GetDataSource(path, VerticalApertureLocator);
  if (verticalApertureSource) {
    auto sampledVerticalApertureSource =
      HdSampledDataSource::Cast(verticalApertureSource);
    auto value = sampledVerticalApertureSource->GetValue(0);
    auto verticalAperture = value.Get<float>();
    diff.add_or_update_camera_vertical_aperture(pathString, verticalAperture);
  }
}

void
CameraObserver::GetDiff(const HdSceneIndexBase& sceneIndex, UsdDataDiff& diff)
{
  // addedされたCameraの情報をdiffに登録する
  for (const auto& path : _added) {
    _UpdateDiff(sceneIndex, diff, path);
  }

  // removedされたCameraの情報をdiffに登録する
  for (const auto& path : _removed) {
    auto pathString = rust::String(path.GetText());
    diff.destroy_camera(pathString);
  }

  // dirtiedされたCameraの情報をdiffに登録する
  for (const auto& path : _dirtied) {
    _UpdateDiff(sceneIndex, diff, path);
  }
}
