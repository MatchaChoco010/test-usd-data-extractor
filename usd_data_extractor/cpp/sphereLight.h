#ifndef BRIDGE_SPHERE_LIGHT_H
#define BRIDGE_SPHERE_LIGHT_H

#include "bridgeSender.h"
#include "pxr/imaging/hd/changeTracker.h"
#include "pxr/imaging/hd/light.h"
#include "pxr/pxr.h"
#include "rust/cxx.h"
#include <iostream>

using namespace pxr;

class HdBridgeSphereLight final : public HdLight
{
public:
  HdBridgeSphereLight(SdfPath const& id, BridgeSenderSharedPtr sender);
  ~HdBridgeSphereLight() override;

  HdDirtyBits GetInitialDirtyBitsMask() const override;

  void Sync(HdSceneDelegate* sceneDelegate,
            HdRenderParam* renderParam,
            HdDirtyBits* dirtyBits) override;

private:
  SdfPath _id;
  BridgeSenderSharedPtr _sender;

  void _SyncTransform(HdSceneDelegate* sceneDelegate);
  void _SyncSphereLightData(HdSceneDelegate* sceneDelegate);

  // This class does not support copying.
  HdBridgeSphereLight(const HdBridgeSphereLight&) = delete;
  HdBridgeSphereLight& operator=(const HdBridgeSphereLight&) = delete;
};

#endif
