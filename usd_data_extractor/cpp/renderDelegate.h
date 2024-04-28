#ifndef BRIDGE_RENDER_DELEGATE_H
#define BRIDGE_RENDER_DELEGATE_H

#include "bridgeSender.h"
#include "camera.h"
#include "distantLight.h"
#include "mesh.h"
#include "pxr/base/tf/diagnostic.h"
#include "pxr/base/tf/staticTokens.h"
#include "pxr/imaging/hd/bprim.h"
#include "pxr/imaging/hd/camera.h"
#include "pxr/imaging/hd/material.h"
#include "pxr/imaging/hd/renderDelegate.h"
#include "pxr/imaging/hd/resourceRegistry.h"
#include "pxr/imaging/hd/rprim.h"
#include "pxr/imaging/hd/sprim.h"
#include "pxr/pxr.h"
#include "rust/cxx.h"
#include "sphereLight.h"
#include <iostream>
#include <memory>

using namespace pxr;

class HdBridgeRenderDelegate final : public HdRenderDelegate
{
public:
  HdBridgeRenderDelegate(BridgeSenderSharedPtr sender);
  HdBridgeRenderDelegate(HdRenderSettingsMap const& settingsMap,
                         BridgeSenderSharedPtr sender);
  virtual ~HdBridgeRenderDelegate() override = default;

  const TfTokenVector& GetSupportedRprimTypes() const override;
  const TfTokenVector& GetSupportedSprimTypes() const override;
  const TfTokenVector& GetSupportedBprimTypes() const override;

  HdResourceRegistrySharedPtr GetResourceRegistry() const override;

  HdRenderPassSharedPtr CreateRenderPass(
    HdRenderIndex* index,
    HdRprimCollection const& collection) override;

  HdInstancer* CreateInstancer(HdSceneDelegate* delegate,
                               SdfPath const& id) override;
  void DestroyInstancer(HdInstancer* instancer) override;

  HdRprim* CreateRprim(TfToken const& typeId, SdfPath const& rprimId) override;
  void DestroyRprim(HdRprim* rPrim) override;

  HdSprim* CreateSprim(TfToken const& typeId, SdfPath const& sprimId) override;
  HdSprim* CreateFallbackSprim(TfToken const& typeId) override;
  void DestroySprim(HdSprim* sprim) override;

  HdBprim* CreateBprim(TfToken const& typeId, SdfPath const& bprimId) override;
  HdBprim* CreateFallbackBprim(TfToken const& typeId) override;
  void DestroyBprim(HdBprim* bprim) override;

  void CommitResources(HdChangeTracker* tracker) override;

  HdRenderParam* GetRenderParam() const override;

private:
  BridgeSenderSharedPtr _sender;

  void _Initialize();

  // This class does not support copying.
  HdBridgeRenderDelegate(const HdBridgeRenderDelegate&) = delete;
  HdBridgeRenderDelegate& operator=(const HdBridgeRenderDelegate&) = delete;
};

#endif
