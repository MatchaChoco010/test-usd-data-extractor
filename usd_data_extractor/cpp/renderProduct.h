#ifndef BRIDGE_RENDER_PRODUCT_H
#define BRIDGE_RENDER_PRODUCT_H

#include "bridgeSender.h"
#include "pxr/base/tf/token.h"
#include "pxr/imaging/hd/changeTracker.h"
#include "pxr/imaging/hd/sprim.h"
#include "pxr/pxr.h"
#include "rust/cxx.h"
#include <iostream>

using namespace pxr;

PXR_NAMESPACE_OPEN_SCOPE

TF_DECLARE_PUBLIC_TOKENS(HdBridgeRenderProductTokens, HD_API, (renderProduct));

PXR_NAMESPACE_CLOSE_SCOPE

class HdBridgeRenderProduct final : public HdSprim
{
public:
  HdBridgeRenderProduct(SdfPath const& id, BridgeSenderSharedPtr sender);
  ~HdBridgeRenderProduct() override;

  HdDirtyBits GetInitialDirtyBitsMask() const override;

  void Sync(HdSceneDelegate* sceneDelegate,
            HdRenderParam* renderParam,
            HdDirtyBits* dirtyBits) override;

private:
  SdfPath _id;
  BridgeSenderSharedPtr _sender;

  void _SyncCameraPath(HdSceneDelegate* sceneDelegate);

  // This class does not support copying.
  HdBridgeRenderProduct(const HdBridgeRenderProduct&) = delete;
  HdBridgeRenderProduct& operator=(const HdBridgeRenderProduct&) = delete;
};

#endif
