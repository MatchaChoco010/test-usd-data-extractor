#ifndef BRIDGE_CAMERA_H
#define BRIDGE_CAMERA_H

#include "bridgeSender.h"
#include "pxr/imaging/hd/camera.h"
#include "pxr/imaging/hd/changeTracker.h"
#include "pxr/pxr.h"
#include "rust/cxx.h"
#include <iostream>

using namespace pxr;

class HdBridgeCamera final : public HdCamera
{
public:
  HdBridgeCamera(SdfPath const& id, BridgeSenderSharedPtr sender);
  ~HdBridgeCamera() override;

  HdDirtyBits GetInitialDirtyBitsMask() const override;

  void Sync(HdSceneDelegate* sceneDelegate,
            HdRenderParam* renderParam,
            HdDirtyBits* dirtyBits) override;

private:
  SdfPath _id;
  BridgeSenderSharedPtr _sender;

  void _SyncTransform(HdSceneDelegate* sceneDelegate);
  void _SyncCameraData(HdSceneDelegate* sceneDelegate);

  // This class does not support copying.
  HdBridgeCamera(const HdBridgeCamera&) = delete;
  HdBridgeCamera& operator=(const HdBridgeCamera&) = delete;
};

#endif
