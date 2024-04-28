#ifndef BRIDGE_DISTANT_LIGHT_H
#define BRIDGE_DISTANT_LIGHT_H

#include "bridgeSender.h"
#include "pxr/imaging/hd/changeTracker.h"
#include "pxr/imaging/hd/light.h"
#include "pxr/pxr.h"
#include "rust/cxx.h"
#include <iostream>

using namespace pxr;

class HdBridgeDistantLight final : public HdLight
{
public:
  HdBridgeDistantLight(SdfPath const& id, BridgeSenderSharedPtr sender);
  ~HdBridgeDistantLight() override;

  HdDirtyBits GetInitialDirtyBitsMask() const override;

  void Sync(HdSceneDelegate* sceneDelegate,
            HdRenderParam* renderParam,
            HdDirtyBits* dirtyBits) override;

private:
  SdfPath _id;
  BridgeSenderSharedPtr _sender;

  void _SyncTransform(HdSceneDelegate* sceneDelegate);
  void _SyncDistantLightData(HdSceneDelegate* sceneDelegate);

  // This class does not support copying.
  HdBridgeDistantLight(const HdBridgeDistantLight&) = delete;
  HdBridgeDistantLight& operator=(const HdBridgeDistantLight&) = delete;
};

#endif
