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
  (*_sender)->send_string(
    rust::String(std::string("=> destroy mesh! : ") + _id.GetText()));
}

HdDirtyBits
HdBridgeMesh::GetInitialDirtyBitsMask() const
{
  HdDirtyBits mask =
    HdChangeTracker::Clean | HdChangeTracker::InitRepr |
    HdChangeTracker::DirtyCullStyle | HdChangeTracker::DirtyDoubleSided |
    HdChangeTracker::DirtyExtent | HdChangeTracker::DirtyNormals |
    HdChangeTracker::DirtyPoints | HdChangeTracker::DirtyPrimID |
    HdChangeTracker::DirtyPrimvar | HdChangeTracker::DirtyDisplayStyle |
    HdChangeTracker::DirtyRepr | HdChangeTracker::DirtyMaterialId |
    HdChangeTracker::DirtyTopology | HdChangeTracker::DirtyTransform |
    HdChangeTracker::DirtyVisibility | HdChangeTracker::DirtyInstancer;
  ;
  return mask;
}

void
HdBridgeMesh::Sync(HdSceneDelegate* sceneDelegate,
                   HdRenderParam* renderParam,
                   HdDirtyBits* dirtyBits,
                   TfToken const& reprToken)
{

  (*_sender)->send_string(
    rust::String(std::string("=> sync mesh! : ") + _id.GetText()));

  if (*dirtyBits & HdChangeTracker::InitRepr) {
    (*_sender)->send_string(rust::String("=> dirty init repr!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyCullStyle) {
    (*_sender)->send_string(rust::String("=> dirty cull style!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyDoubleSided) {
    (*_sender)->send_string(rust::String("=> dirty double sided!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyExtent) {
    (*_sender)->send_string(rust::String("=> dirty extent!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyNormals) {
    (*_sender)->send_string(rust::String("=> dirty normals!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyPoints) {
    (*_sender)->send_string(rust::String("=> dirty points!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyPrimID) {
    (*_sender)->send_string(rust::String("=> dirty prim id!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyPrimvar) {
    (*_sender)->send_string(rust::String("=> dirty prim var!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyDisplayStyle) {
    (*_sender)->send_string(rust::String("=> dirty display style!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyRepr) {
    (*_sender)->send_string(rust::String("=> dirty repr!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyMaterialId) {
    (*_sender)->send_string(rust::String("=> dirty material id!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyTopology) {
    (*_sender)->send_string(rust::String("=> dirty topology!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyTransform) {
    (*_sender)->send_string(rust::String("=> dirty transform!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyVisibility) {
    (*_sender)->send_string(rust::String("=> dirty visibility!"));
  }

  if (*dirtyBits & HdChangeTracker::DirtyInstancer) {
    (*_sender)->send_string(rust::String("=> dirty instancer!"));
  }

  *dirtyBits = HdChangeTracker::Clean;
}

void
HdBridgeMesh::_InitRepr(TfToken const& reprToken, HdDirtyBits* dirtyBits)
{
  (*_sender)->send_string(
    rust::String(std::string("=> init repr! : ") + _id.GetText()));
}

HdDirtyBits
HdBridgeMesh::_PropagateDirtyBits(HdDirtyBits bits) const
{
  return bits;
}
