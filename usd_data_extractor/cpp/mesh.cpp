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
    _SyncIndices(sceneDelegate);
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
  VtValue value = sceneDelegate->Get(_id, HdTokens->points);
  if (value.IsEmpty()) {
    return;
  }
  if (!value.IsHolding<VtVec3fArray>()) {
    return;
  }
  VtVec3fArray points = value.Get<VtVec3fArray>();
  rust::Slice<const float> pointsSlice{ (const float*)points.data(),
                                        points.size() * 3 };

  // interpolation typeを取得
  std::vector<HdPrimvarDescriptor> primvarDescs =
    sceneDelegate->GetPrimvarDescriptors(_id, HdInterpolationVertex);
  uint8_t interpolation = 255;
  for (const HdPrimvarDescriptor& desc : primvarDescs) {
    if (desc.name == HdTokens->points) {
      interpolation = (uint8_t)desc.interpolation;
      break;
    }
  }
  if (interpolation == 255) {
    interpolation = (uint8_t)HdInterpolationFaceVarying;
  }

  (*_sender)->points(path, pointsSlice, interpolation);
}

void
HdBridgeMesh::_SyncNormals(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());

  // normalsを取得
  VtValue value = sceneDelegate->Get(_id, HdTokens->normals);
  if (value.IsEmpty()) {
    return;
  }
  if (!value.IsHolding<VtVec3fArray>()) {
    return;
  }
  VtVec3fArray normals = value.Get<VtVec3fArray>();
  rust::Slice<const float> normalsSlice{ (const float*)normals.data(),
                                         normals.size() * 3 };

  // interpolation typeを取得
  std::vector<HdPrimvarDescriptor> primvarDescs =
    sceneDelegate->GetPrimvarDescriptors(_id, HdInterpolationVertex);
  uint8_t interpolation = 255;
  for (const HdPrimvarDescriptor& desc : primvarDescs) {
    if (desc.name == HdTokens->normals) {
      interpolation = (uint8_t)desc.interpolation;
      break;
    }
  }
  if (interpolation == 255) {
    interpolation = (uint8_t)HdInterpolationFaceVarying;
  }

  (*_sender)->normals(path, normalsSlice, interpolation);
}

void
HdBridgeMesh::_SyncUVs(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());

  // uvsを取得
  TfToken uvPrimvarName("st");
  VtValue value = sceneDelegate->Get(_id, uvPrimvarName);
  if (value.IsEmpty()) {
    return;
  }
  if (!value.IsHolding<VtVec2fArray>()) {
    return;
  }
  VtVec2fArray uvs = value.Get<VtVec2fArray>();
  rust::Slice<const float> uvsSlice{ (const float*)uvs.data(), uvs.size() * 2 };

  // interpolation typeを取得
  std::vector<HdPrimvarDescriptor> primvarDescs =
    sceneDelegate->GetPrimvarDescriptors(_id, HdInterpolationVertex);
  uint8_t interpolation = 255;
  for (const HdPrimvarDescriptor& desc : primvarDescs) {
    if (desc.name == uvPrimvarName) {
      interpolation = (uint8_t)desc.interpolation;
      break;
    }
  }
  if (interpolation == 255) {
    interpolation = (uint8_t)HdInterpolationFaceVarying;
  }

  (*_sender)->uvs(path, uvsSlice, interpolation);
}

void
HdBridgeMesh::_SyncIndices(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());

  HdMeshTopology topology = sceneDelegate->GetMeshTopology(_id);
  const VtIntArray& faceVertexIndices = topology.GetFaceVertexIndices();

  rust::Slice<const int> faceVertexIndicesSlice{
    (const int*)faceVertexIndices.data(), faceVertexIndices.size()
  };

  (*_sender)->indices(path, faceVertexIndicesSlice);
}
