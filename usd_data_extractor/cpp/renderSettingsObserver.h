#ifndef RENDER_SETTINGS_OBSERVER_H
#define RENDER_SETTINGS_OBSERVER_H

#include "pxr/imaging/hd/dataSource.h"
#include "pxr/imaging/hd/sceneIndexObserver.h"
#include "pxr/pxr.h"
#include "pxr/usd/sdf/path.h"
#include "usdDataDiff.h"
#include <iostream>
#include <set>

using namespace pxr;

// primTypeがRenderSettingsの情報を処理してRustにdiffを受け渡すためのクラス。
class RenderSettingsObserver
{

public:
  RenderSettingsObserver();
  virtual ~RenderSettingsObserver();

  inline static const TfToken TypeToken = TfToken("renderSettings");

  inline static const HdDataSourceLocator RenderProductsLocator =
    HdDataSourceLocator(TfToken("renderSettings"), TfToken("renderProducts"));

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
  // stageに存在するRenderSettingsのPathを記録する
  std::set<SdfPath> _lightPaths;

  // 前回GetDiffしてClearしてから追加されたRenderSettingsの差分のPathを記録する
  std::set<SdfPath> _added;
  // 前回GetDiffしてClearしてから削除されたRenderSettingsのPathを記録する
  std::set<SdfPath> _removed;
  // 前回までにGetDiffで追加されたRenderSettingsを記録する
  std::set<SdfPath> _dirtied;

  void _UpdateDiff(const HdSceneIndexBase& sceneIndex,
                   UsdDataDiff& diff,
                   const SdfPath path) const;

  // This class does not support copying.
  RenderSettingsObserver(const RenderSettingsObserver&) = delete;
  RenderSettingsObserver& operator=(const RenderSettingsObserver&) = delete;
};

#endif
