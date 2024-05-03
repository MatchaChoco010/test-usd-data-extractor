#ifndef MATERIAL_OBSERVER_H
#define MATERIAL_OBSERVER_H

#include "pxr/imaging/hd/dataSource.h"
#include "pxr/imaging/hd/sceneIndexObserver.h"
#include "pxr/pxr.h"
#include "pxr/usd/sdf/assetPath.h"
#include "pxr/usd/sdf/path.h"
#include "usdDataDiff.h"
#include <iostream>
#include <optional>
#include <set>

using namespace pxr;

// primTypeがMaterialの情報を処理してRustにdiffを受け渡すためのクラス。
class MaterialObserver
{

public:
  MaterialObserver();
  virtual ~MaterialObserver();

  inline static const TfToken TypeToken = TfToken("material");
  inline static const TfToken UsdPreviewSurfaceToken =
    TfToken("UsdPreviewSurface");

  inline static const HdDataSourceLocator TerminalNodePathLocator =
    HdDataSourceLocator(TfToken("material"),
                        TfToken(""),
                        TfToken("terminals"),
                        TfToken("surface"),
                        TfToken("upstreamNodePath"));

  inline static const HdDataSourceLocator NodesLocator =
    HdDataSourceLocator(TfToken("material"), TfToken(""), TfToken("nodes"));
  inline static const HdDataSourceLocator NodeIdentifierLocator =
    HdDataSourceLocator(TfToken("nodeIdentifier"));

  inline static const HdDataSourceLocator DiffuseColorParameterLocator =
    HdDataSourceLocator(TfToken("parameters"),
                        TfToken("diffuseColor"),
                        TfToken("value"));
  inline static const HdDataSourceLocator EmissiveParameterLocator =
    HdDataSourceLocator(TfToken("parameters"),
                        TfToken("emissive"),
                        TfToken("value"));
  inline static const HdDataSourceLocator MetallicParameterLocator =
    HdDataSourceLocator(TfToken("parameters"),
                        TfToken("metallic"),
                        TfToken("value"));
  inline static const HdDataSourceLocator OpacityParameterLocator =
    HdDataSourceLocator(TfToken("parameters"),
                        TfToken("opacity"),
                        TfToken("value"));
  inline static const HdDataSourceLocator RoughnessParameterLocator =
    HdDataSourceLocator(TfToken("parameters"),
                        TfToken("roughness"),
                        TfToken("value"));

  inline static const HdDataSourceLocator DiffuseColorConnectionLocator =
    HdDataSourceLocator(TfToken("inputConnections"), TfToken("diffuseColor"));
  inline static const HdDataSourceLocator EmissiveConnectionLocator =
    HdDataSourceLocator(TfToken("inputConnections"), TfToken("emissive"));
  inline static const HdDataSourceLocator MetallicConnectionLocator =
    HdDataSourceLocator(TfToken("inputConnections"), TfToken("metallic"));
  inline static const HdDataSourceLocator NormalConnectionLocator =
    HdDataSourceLocator(TfToken("inputConnections"), TfToken("normal"));
  inline static const HdDataSourceLocator OpacityConnectionLocator =
    HdDataSourceLocator(TfToken("inputConnections"), TfToken("opacity"));
  inline static const HdDataSourceLocator RoughnessConnectionLocator =
    HdDataSourceLocator(TfToken("inputConnections"), TfToken("roughness"));
  inline static const TfToken UpstreamNodePathToken =
    TfToken("upstreamNodePath");

  inline static const HdDataSourceLocator FileParameterLocator =
    HdDataSourceLocator(TfToken("parameters"),
                        TfToken("file"),
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
  // stageに存在するMaterialのPathを記録する
  std::set<SdfPath> _materialPaths;

  // 前回GetDiffしてClearしてから追加されたMaterialの差分のPathを記録する
  std::set<SdfPath> _added;
  // 前回GetDiffしてClearしてから削除されたMaterialのPathを記録する
  std::set<SdfPath> _removed;
  // 前回までにGetDiffで追加されたMaterialを記録する
  std::set<SdfPath> _dirtied;

  std::optional<rust::String> _GetMaterialFilePath(
    const HdSceneIndexBase& sceneIndex,
    const SdfPath& path,
    const HdDataSourceLocator connectionLocator) const;

  void _UpdateDiff(const HdSceneIndexBase& sceneIndex,
                   UsdDataDiff& diff,
                   const SdfPath path) const;

  // This class does not support copying.
  MaterialObserver(const MaterialObserver&) = delete;
  MaterialObserver& operator=(const MaterialObserver&) = delete;
};

#endif
