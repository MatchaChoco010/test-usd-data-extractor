#ifndef BRIDGE_MESH_H
#define BRIDGE_MESH_H

#include "bridgeSender.h"
#include "pxr/imaging/hd/changeTracker.h"
#include "pxr/imaging/hd/mesh.h"
#include "pxr/pxr.h"
#include "rust/cxx.h"

using namespace pxr;

class HdBridgeMesh final : public HdMesh
{
public:
  HdBridgeMesh(SdfPath const& id, BridgeSenderSharedPtr sender);
  ~HdBridgeMesh() override;

  HdDirtyBits GetInitialDirtyBitsMask() const override;

  void Sync(HdSceneDelegate* sceneDelegate,
            HdRenderParam* renderParam,
            HdDirtyBits* dirtyBits,
            TfToken const& reprToken) override;

protected:
  void _InitRepr(TfToken const& reprToken, HdDirtyBits* dirtyBits) override;
  HdDirtyBits _PropagateDirtyBits(HdDirtyBits bits) const override;

private:
  BridgeSenderSharedPtr _sender;

  // This class does not support copying.
  HdBridgeMesh(const HdBridgeMesh&) = delete;
  HdBridgeMesh& operator=(const HdBridgeMesh&) = delete;
};

#endif