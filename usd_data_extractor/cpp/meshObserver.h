#ifndef MESH_OBSERVER_H
#define MESH_OBSERVER_H

#include "pxr/imaging/hd/dataSource.h"
#include "pxr/imaging/hd/sceneIndexObserver.h"
#include "pxr/pxr.h"
#include "pxr/usd/sdf/path.h"
#include "usdDataDiff.h"
#include <iostream>
#include <map>
#include <set>

using namespace pxr;

// Locatorは細かいprimvar単位などで変更通知を受け取れるが、
// Rust側にはMeshDataの一部に変更があったらMeshDataの情報全体を渡しているので、
// Rustとの同期の単位はTransformMatrixかMeshDataかの二択。
// MeshDataを全部一括で渡すのは、Rust側でメッシュの頂点のduplicate処理とかをして
// 頂点バッファを構築し直すのに一通りの情報が必要なため。
enum class DiffType
{
  TransformMatrix,
  MeshData,
};

// primTypeがMeshの情報を処理してRustにdiffを受け渡すためのクラス。
class MeshObserver
{

public:
  MeshObserver();
  virtual ~MeshObserver();

  inline static const TfToken TypeToken = TfToken("mesh");

  inline static const HdDataSourceLocator TransforLocator =
    HdDataSourceLocator(TfToken("xform"));
  inline static const HdDataSourceLocator PrimvarsLocator =
    HdDataSourceLocator(TfToken("privars"));
  inline static const HdDataSourceLocator MaterialBindingsLocator =
    HdDataSourceLocator(TfToken("materialBindings"));
  inline static const HdDataSourceLocator MeshLocator =
    HdDataSourceLocator(TfToken("mesh"));

  inline static const HdDataSourceLocator TransformMatrixDataLocator =
    HdDataSourceLocator(TfToken("xform"), TfToken("matrix"));
  inline static const HdDataSourceLocator LeftHandedDataLocator =
    HdDataSourceLocator(TfToken("mesh"),
                        TfToken("topology"),
                        TfToken("orientation"));
  inline static const HdDataSourceLocator PointsDataLocator =
    HdDataSourceLocator(TfToken("primvars"),
                        TfToken("points"),
                        TfToken("primvarValue"));
  inline static const HdDataSourceLocator NormalsDataLocator =
    HdDataSourceLocator(TfToken("primvars"),
                        TfToken("normals"),
                        TfToken("primvarValue"));
  inline static const HdDataSourceLocator NormalsInterpolationDataLocator =
    HdDataSourceLocator(TfToken("primvars"),
                        TfToken("normals"),
                        TfToken("interpolation"));
  inline static const HdDataSourceLocator UVsDataLocator =
    HdDataSourceLocator(TfToken("primvars"),
                        TfToken("uv"),
                        TfToken("primvarValue"));
  inline static const HdDataSourceLocator UVsInterpolationDataLocator =
    HdDataSourceLocator(TfToken("primvars"),
                        TfToken("uv"),
                        TfToken("interpolation"));
  inline static const HdDataSourceLocator FaceVertexIndicesLocator =
    HdDataSourceLocator(TfToken("mesh"),
                        TfToken("topology"),
                        TfToken("faceVertexIndices"));
  inline static const HdDataSourceLocator FaceVertexCountsLocator =
    HdDataSourceLocator(TfToken("mesh"),
                        TfToken("topology"),
                        TfToken("faceVertexCounts"));
  inline static const HdDataSourceLocator GeomSubsetLocator =
    HdDataSourceLocator(TfToken("mesh"), TfToken("geomSubsets"));

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
  // stageに存在するMeshのPathを記録する
  std::set<SdfPath> _meshPaths;

  // 前回GetDiffしてClearしてから追加されたMeshの差分のPathを記録する
  std::set<SdfPath> _added;
  // 前回GetDiffしてClearしてから削除されたMeshのPathを記録する
  std::set<SdfPath> _removed;
  // 前回までにGetDiffで追加されたものの情報の更新の場合を記録する
  std::map<SdfPath, std::set<DiffType>> _dirtied;

  // This class does not support copying.
  MeshObserver(const MeshObserver&) = delete;
  MeshObserver& operator=(const MeshObserver&) = delete;
};

#endif
