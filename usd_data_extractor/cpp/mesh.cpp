#include "mesh.h"
#include "usd_data_extractor/src/bridge.rs.h"

HdBridgeMesh::HdBridgeMesh(SdfPath const& id, BridgeSenderSharedPtr sender)
  : HdMesh(id)
  , _id(id)
  , _sender(sender)
{
  rust::String path = rust::string(this->_id.GetText());
  (*_sender)->create_mesh(path);
}

HdBridgeMesh::~HdBridgeMesh()
{
  rust::String path = rust::string(this->_id.GetText());
  (*_sender)->destroy_mesh(path);
}

HdDirtyBits
HdBridgeMesh::GetInitialDirtyBitsMask() const
{
  HdDirtyBits mask =
    HdChangeTracker::Clean | HdChangeTracker::DirtyNormals |
    HdChangeTracker::DirtyPoints | HdChangeTracker::DirtyPrimvar |
    HdChangeTracker::DirtyMaterialId | HdChangeTracker::DirtyTopology |
    HdChangeTracker::DirtyTransform;
  // | HdChangeTracker::DirtyVisibility | HdChangeTracker::DirtyInstancer;
  return mask;
}

void
HdBridgeMesh::Sync(HdSceneDelegate* sceneDelegate,
                   HdRenderParam* renderParam,
                   HdDirtyBits* dirtyBits,
                   TfToken const& reprToken)
{
  if (*dirtyBits & HdChangeTracker::DirtyNormals) {
    _SyncNormals(sceneDelegate);
  }

  if (*dirtyBits & HdChangeTracker::DirtyPoints) {
    _SyncPoints(sceneDelegate);
  }

  if (*dirtyBits & HdChangeTracker::DirtyPrimvar) {
    _SyncUVs(sceneDelegate);
  }

  if (*dirtyBits & HdChangeTracker::DirtyMaterialId) {
    (*_sender)->message(rust::String("=> dirty material id!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyTopology) {
    _SyncTopology(sceneDelegate);
  }

  if (*dirtyBits & HdChangeTracker::DirtyTransform) {
    _SyncTransform(sceneDelegate);
  }

  *dirtyBits = HdChangeTracker::Clean;
}

void
HdBridgeMesh::_InitRepr(TfToken const& reprToken, HdDirtyBits* dirtyBits)
{
}

HdDirtyBits
HdBridgeMesh::_PropagateDirtyBits(HdDirtyBits bits) const
{
  return bits;
}

void
HdBridgeMesh::_SyncTransform(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());

  GfMatrix4d matrix = sceneDelegate->GetTransform(_id);
  const double* data = matrix.GetArray();
  rust::Slice<const double> dataSlice{ data, 16 };

  (*_sender)->transform_matrix(path, dataSlice);
}

void
HdBridgeMesh::_SyncPoints(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());

  // pointsを取得
  std::vector<HdPrimvarDescriptor> primvarDescs =
    sceneDelegate->GetPrimvarDescriptors(_id, HdInterpolationVertex);
  for (const HdPrimvarDescriptor& desc : primvarDescs) {
    if (desc.name == HdTokens->points) {

      VtValue value = sceneDelegate->Get(_id, desc.name);
      if (!value.IsEmpty() && value.IsHolding<VtVec3fArray>()) {
        VtVec3fArray points = value.Get<VtVec3fArray>();
        rust::Slice<const float> pointsSlice{ (const float*)points.data(),
                                              points.size() * 3 };

        (*_sender)->points(path, pointsSlice, (uint8_t)desc.interpolation);
      }

      break;
    }
  }
}

void
HdBridgeMesh::_SyncNormals(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());

  // normalsを取得
  std::vector<HdPrimvarDescriptor> primvarDescs;
  {
    std::vector<HdPrimvarDescriptor> a =
      sceneDelegate->GetPrimvarDescriptors(_id, HdInterpolationVertex);
    primvarDescs.insert(primvarDescs.end(), a.begin(), a.end());
    std::vector<HdPrimvarDescriptor> b =
      sceneDelegate->GetPrimvarDescriptors(_id, HdInterpolationFaceVarying);
    primvarDescs.insert(primvarDescs.end(), b.begin(), b.end());
  }
  for (const HdPrimvarDescriptor& desc : primvarDescs) {
    if (desc.name == HdTokens->normals) {

      VtValue value = sceneDelegate->Get(_id, desc.name);
      if (!value.IsEmpty() && value.IsHolding<VtVec3fArray>()) {
        VtVec3fArray normals = value.Get<VtVec3fArray>();
        rust::Slice<const float> normalsSlice{ (const float*)normals.data(),
                                               normals.size() * 3 };

        (*_sender)->normals(path, normalsSlice, (uint8_t)desc.interpolation);
      }

      break;
    }
  }
}

void
HdBridgeMesh::_SyncUVs(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());

  // uvsを取得
  TfToken uvPrimvarName("st");
  std::vector<HdPrimvarDescriptor> primvarDescs =
    sceneDelegate->GetPrimvarDescriptors(_id, HdInterpolationFaceVarying);
  for (const HdPrimvarDescriptor& desc : primvarDescs) {
    if (desc.name == uvPrimvarName) {

      VtValue value = sceneDelegate->Get(_id, desc.name);
      if (!value.IsEmpty() && value.IsHolding<VtVec2fArray>()) {
        VtVec2fArray uvs = value.Get<VtVec2fArray>();
        rust::Slice<const float> uvsSlice{ (const float*)uvs.data(),
                                           uvs.size() * 2 };

        (*_sender)->uvs(path, uvsSlice, (uint8_t)desc.interpolation);
      }

      break;
    }
  }
}

void
HdBridgeMesh::_SyncTopology(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());

  HdMeshTopology topology = sceneDelegate->GetMeshTopology(_id);

  const VtIntArray& faceVertexIndices = topology.GetFaceVertexIndices();
  rust::Slice<const int> faceVertexIndicesSlice{
    (const int*)faceVertexIndices.data(), faceVertexIndices.size()
  };
  (*_sender)->face_vertex_indices(path, faceVertexIndicesSlice);

  const VtIntArray& faceVertexCounts = topology.GetFaceVertexCounts();
  rust::Slice<const int> faceVertexCountsSlice{
    (const int*)faceVertexCounts.data(), faceVertexCounts.size()
  };
  (*_sender)->face_vertex_counts(path, faceVertexCountsSlice);
}
