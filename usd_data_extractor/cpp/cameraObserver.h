#ifndef CAMERA_OBSERVER_H
#define CAMERA_OBSERVER_H

#include "pxr/imaging/hd/dataSource.h"
#include "pxr/imaging/hd/sceneIndexObserver.h"
#include "pxr/pxr.h"
#include "pxr/usd/sdf/path.h"
#include "usdDataDiff.h"
#include <iostream>
#include <set>

using namespace pxr;

// primTypeがCameraの情報を処理してRustにdiffを受け渡すためのクラス。
class CameraObserver
{

public:
  CameraObserver();
  virtual ~CameraObserver();

  inline static const TfToken TypeToken = TfToken("camera");

  inline static const HdDataSourceLocator TransforLocator =
    HdDataSourceLocator(TfToken("xform"));

  inline static const HdDataSourceLocator TransformMatrixLocator =
    HdDataSourceLocator(TfToken("xform"), TfToken("matrix"));
  inline static const HdDataSourceLocator FocalLengthLocator =
    HdDataSourceLocator(TfToken("camera"), TfToken("focalLength"));
  inline static const HdDataSourceLocator VerticalApertureLocator =
    HdDataSourceLocator(TfToken("camera"), TfToken("verticalAperture"));

  void PrimsAdded(const HdSceneIndexBase& sender,
                  const HdSceneIndexObserver::AddedPrimEntries& entries);

  void PrimsRemoved(const HdSceneIndexBase& sender,
                    const HdSceneIndexObserver::RemovedPrimEntries& entries);

  void PrimsDirtied(const HdSceneIndexBase& sender,
                    const HdSceneIndexObserver::DirtiedPrimEntries& entries);

  void PrimsRenamed(const HdSceneIndexBase& sender,
                    const HdSceneIndexObserver::RenamedPrimEntries& entries);

  void ClearDiff();

  void GetDiff(const HdSceneIndexBase& sceneIndex, UsdDataDiff& diff);

private:
  // stageに存在するCameraのPathを記録する
  std::set<SdfPath> _lightPaths;

  // 前回GetDiffしてClearしてから追加されたCameraの差分のPathを記録する
  std::set<SdfPath> _added;
  // 前回GetDiffしてClearしてから削除されたCameraのPathを記録する
  std::set<SdfPath> _removed;
  // 前回までにGetDiffで追加されたCameraを記録する
  std::set<SdfPath> _dirtied;

  void _UpdateDiff(const HdSceneIndexBase& sceneIndex,
                   UsdDataDiff& diff,
                   const SdfPath path) const;

  // This class does not support copying.
  CameraObserver(const CameraObserver&) = delete;
  CameraObserver& operator=(const CameraObserver&) = delete;
};

#endif
