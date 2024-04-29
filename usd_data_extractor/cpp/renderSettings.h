#ifndef BRIDGE_RENDER_SETTINGS_H
#define BRIDGE_RENDER_SETTINGS_H

#include "bridgeSender.h"
#include "pxr/imaging/hd/changeTracker.h"
#include "pxr/imaging/hd/renderSettings.h"
#include "pxr/pxr.h"
#include "rust/cxx.h"
#include <iostream>

using namespace pxr;

class HdBridgeRenderSettings final : public HdRenderSettings
{
public:
  HdBridgeRenderSettings(SdfPath const& id, BridgeSenderSharedPtr sender);
  ~HdBridgeRenderSettings() override;

  HdDirtyBits GetInitialDirtyBitsMask() const override;

protected:
  void _Sync(HdSceneDelegate* sceneDelegate,
             HdRenderParam* renderParam,
             const HdDirtyBits* dirtyBits) override;

private:
  SdfPath _id;
  BridgeSenderSharedPtr _sender;

  void _SyncRenderProducts(HdSceneDelegate* sceneDelegate);

  // This class does not support copying.
  HdBridgeRenderSettings(const HdBridgeRenderSettings&) = delete;
  HdBridgeRenderSettings& operator=(const HdBridgeRenderSettings&) = delete;
};

#endif
