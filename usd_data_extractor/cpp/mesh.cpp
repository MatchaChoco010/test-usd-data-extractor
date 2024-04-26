#include "mesh.h"
#include "usd_data_extractor/src/bridge.rs.h"

HdBridgeMesh::HdBridgeMesh(SdfPath const& id, BridgeSenderSharedPtr sender)
  : HdMesh(id)
  , _id(id)
  , _sender(sender)
{
}

HdBridgeMesh::~HdBridgeMesh()
{
  (*_sender)->message(
    rust::String(std::string("=> destroy mesh! : ") + _id.GetText()));
}

HdDirtyBits
HdBridgeMesh::GetInitialDirtyBitsMask() const
{
  HdDirtyBits mask =
    HdChangeTracker::Clean | HdChangeTracker::InitRepr |
    HdChangeTracker::DirtyRepr | HdChangeTracker::DirtyNormals |
    HdChangeTracker::DirtyPoints | HdChangeTracker::DirtyPrimID |
    HdChangeTracker::DirtyPrimvar | HdChangeTracker::DirtyMaterialId |
    HdChangeTracker::DirtyTopology | HdChangeTracker::DirtyTransform;
  // | HdChangeTracker::DirtyVisibility | HdChangeTracker::DirtyInstancer;
  return mask;
}

void
HdBridgeMesh::Sync(HdSceneDelegate* sceneDelegate,
                   HdRenderParam* renderParam,
                   HdDirtyBits* dirtyBits,
                   TfToken const& reprToken)
{

  (*_sender)->message(
    rust::String(std::string("=> sync mesh! : ") + _id.GetText()));

  if (*dirtyBits & HdChangeTracker::InitRepr) {
    (*_sender)->message(rust::String("=> dirty init repr!"));
  }
  if (*dirtyBits & HdChangeTracker::DirtyRepr) {
    (*_sender)->message(rust::String("=> dirty repr!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyNormals) {
    (*_sender)->message(rust::String("=> dirty normals!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyPoints) {
    (*_sender)->message(rust::String("=> dirty points!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyPrimID) {
    (*_sender)->message(rust::String("=> dirty prim id!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyPrimvar) {
    (*_sender)->message(rust::String("=> dirty prim var!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyMaterialId) {
    (*_sender)->message(rust::String("=> dirty material id!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyTopology) {
    (*_sender)->message(rust::String("=> dirty topology!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyTransform) {
    _SyncTransform(sceneDelegate);
  }

  *dirtyBits = HdChangeTracker::Clean;
}

void
HdBridgeMesh::_InitRepr(TfToken const& reprToken, HdDirtyBits* dirtyBits)
{
  (*_sender)->message(
    rust::String(std::string("=> init repr! : ") + _id.GetText()));
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
