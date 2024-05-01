#ifndef SPHERE_LIGHT_OBSERVER_H
#define SPHERE_LIGHT_OBSERVER_H

#include "pxr/imaging/hd/dataSource.h"
#include "pxr/imaging/hd/sceneIndexObserver.h"
#include "pxr/pxr.h"
#include "pxr/usd/sdf/path.h"
#include "usdDataDiff.h"
#include <iostream>
#include <set>

using namespace pxr;

// primTypeがsphereLightの情報を処理してRustにdiffを受け渡すためのクラス。
class SphereLightObserver
{

public:
  SphereLightObserver();
  virtual ~SphereLightObserver();

  inline static const TfToken TypeToken = TfToken("sphereLight");

  inline static const HdDataSourceLocator TransforLocator =
    HdDataSourceLocator(TfToken("xform"));
  inline static const HdDataSourceLocator MaterialLocator =
    HdDataSourceLocator(TfToken("material"));

  inline static const HdDataSourceLocator TransformMatrixLocator =
    HdDataSourceLocator(TfToken("xform"), TfToken("matrix"));
  inline static const HdDataSourceLocator MaterialTerminalLocator =
    HdDataSourceLocator(TfToken("material"),
                        TfToken(""),
                        TfToken("terminals"),
                        TfToken("light"),
                        TfToken("upstreamNodePath"));
  inline static const HdDataSourceLocator MaterialNodesLocator =
    HdDataSourceLocator(TfToken("material"), TfToken(""), TfToken("nodes"));
  inline static const HdDataSourceLocator ColorParameterLocator =
    HdDataSourceLocator(TfToken("parameters"),
                        TfToken("color"),
                        TfToken("value"));
  inline static const HdDataSourceLocator IntensityParameterLocator =
    HdDataSourceLocator(TfToken("parameters"),
                        TfToken("intensity"),
                        TfToken("value"));
  inline static const HdDataSourceLocator AngleParameterLocator =
    HdDataSourceLocator(TfToken("parameters"),
                        TfToken("shaping:cone:angle"),
                        TfToken("value"));
  inline static const HdDataSourceLocator SoftnessParameterLocator =
    HdDataSourceLocator(TfToken("parameters"),
                        TfToken("shaping:cone:softness"),
                        TfToken("value"));

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
  // stageに存在するSphereLightのPathを記録する
  std::set<SdfPath> _lightPaths;

  // 前回GetDiffしてClearしてから追加されたSphereLightの差分のPathを記録する
  std::set<SdfPath> _added;
  // 前回GetDiffしてClearしてから削除されたSphereLightのPathを記録する
  std::set<SdfPath> _removed;
  // 前回までにGetDiffで追加されたSphereLightを記録する
  std::set<SdfPath> _dirtied;

  // This class does not support copying.
  SphereLightObserver(const SphereLightObserver&) = delete;
  SphereLightObserver& operator=(const SphereLightObserver&) = delete;
};

#endif
