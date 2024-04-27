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
  if (*dirtyBits &
      (HdChangeTracker::DirtyNormals | HdChangeTracker::DirtyPoints |
       HdChangeTracker::DirtyPrimvar | HdChangeTracker::DirtyTopology)) {
    _SyncMeshData(sceneDelegate);
  }

  if (*dirtyBits & HdChangeTracker::DirtyMaterialId) {
    (*_sender)->message(rust::String("=> dirty material id!"));
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
HdBridgeMesh::_SyncMeshData(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());
  rust::Box<MeshData> meshData = new_mesh_data();

  // check left-handed or right-handed
  {
    TfToken orientation("orientation");
    VtValue orientationValue = sceneDelegate->Get(_id, orientation);
    if (!orientationValue.IsEmpty() && orientationValue.IsHolding<TfToken>()) {
      TfToken orientation = orientationValue.Get<TfToken>();
      if (orientation == HdTokens->leftHanded) {
        meshData->set_left_handed(true);
      }
    }
  }

  // primvars
  TfToken uvPrimvarName("st");
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
    // points
    if (desc.name == HdTokens->points) {
      VtValue value = sceneDelegate->Get(_id, desc.name);
      if (!value.IsEmpty() && value.IsHolding<VtVec3fArray>()) {
        VtVec3fArray points = value.Get<VtVec3fArray>();
        rust::Slice<const float> pointsSlice{ (const float*)points.data(),
                                              points.size() * 3 };
        meshData->set_points(pointsSlice, (uint8_t)desc.interpolation);
      }
    }

    // normals
    if (desc.name == HdTokens->normals) {
      VtValue value = sceneDelegate->Get(_id, desc.name);
      if (!value.IsEmpty() && value.IsHolding<VtVec3fArray>()) {
        VtVec3fArray normals = value.Get<VtVec3fArray>();
        rust::Slice<const float> normalsSlice{ (const float*)normals.data(),
                                               normals.size() * 3 };
        meshData->set_normals(normalsSlice, (uint8_t)desc.interpolation);
      }
    }

    // uvs
    if (desc.name == uvPrimvarName) {
      VtValue value = sceneDelegate->Get(_id, desc.name);
      if (!value.IsEmpty() && value.IsHolding<VtVec2fArray>()) {
        VtVec2fArray uvs = value.Get<VtVec2fArray>();
        rust::Slice<const float> uvsSlice{ (const float*)uvs.data(),
                                           uvs.size() * 2 };
        meshData->set_uvs(uvsSlice, (uint8_t)desc.interpolation);
      }
    }
  }

  // topology
  {
    HdMeshTopology topology = sceneDelegate->GetMeshTopology(_id);

    const VtIntArray& faceVertexIndices = topology.GetFaceVertexIndices();
    rust::Slice<const int> faceVertexIndicesSlice{
      (const int*)faceVertexIndices.data(), faceVertexIndices.size()
    };
    meshData->set_face_vertex_indices(faceVertexIndicesSlice);

    const VtIntArray& faceVertexCounts = topology.GetFaceVertexCounts();
    rust::Slice<const int> faceVertexCountsSlice{
      (const int*)faceVertexCounts.data(), faceVertexCounts.size()
    };
    meshData->set_face_vertex_counts(faceVertexCountsSlice);
  }

  (*_sender)->mesh_data(path, std::move(meshData));
}
