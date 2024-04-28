#include "renderDelegate.h"
#include "usd_data_extractor/src/bridge.rs.h"

TfTokenVector SUPPORTED_RPRIM_TYPES = {
  HdPrimTypeTokens->mesh,
};
TfTokenVector SUPPORTED_SPRIM_TYPES = {
  HdPrimTypeTokens->camera,      HdPrimTypeTokens->material,
  HdPrimTypeTokens->light,       HdPrimTypeTokens->distantLight,
  HdPrimTypeTokens->sphereLight,
};
TfTokenVector SUPPORTED_BPRIM_TYPES = {};

HdBridgeRenderDelegate::HdBridgeRenderDelegate(BridgeSenderSharedPtr sender)
  : HdRenderDelegate()
  , _sender(sender)
{
  _Initialize();
}

HdBridgeRenderDelegate::HdBridgeRenderDelegate(
  HdRenderSettingsMap const& settingsMap,
  BridgeSenderSharedPtr sender)
  : HdRenderDelegate(settingsMap)
  , _sender(sender)
{
  _Initialize();
}

void
HdBridgeRenderDelegate::_Initialize()
{
}

TfTokenVector const&
HdBridgeRenderDelegate::GetSupportedRprimTypes() const
{
  return SUPPORTED_RPRIM_TYPES;
}

TfTokenVector const&
HdBridgeRenderDelegate::GetSupportedSprimTypes() const
{
  return SUPPORTED_SPRIM_TYPES;
}

TfTokenVector const&
HdBridgeRenderDelegate::GetSupportedBprimTypes() const
{
  return SUPPORTED_BPRIM_TYPES;
}

HdResourceRegistrySharedPtr
HdBridgeRenderDelegate::GetResourceRegistry() const
{
  return nullptr;
}

void
HdBridgeRenderDelegate::CommitResources(HdChangeTracker* tracker)
{
}

HdRenderPassSharedPtr
HdBridgeRenderDelegate::CreateRenderPass(HdRenderIndex* index,
                                         HdRprimCollection const& collection)
{
  return nullptr;
}

HdRprim*
HdBridgeRenderDelegate::CreateRprim(TfToken const& typeId,
                                    SdfPath const& rprimId)
{
  if (typeId == HdPrimTypeTokens->mesh) {
    return new HdBridgeMesh(rprimId, _sender);
  }

  TF_CODING_ERROR(
    "Unknown Rprim type=%s id=%s", typeId.GetText(), rprimId.GetText());

  return nullptr;
}

void
HdBridgeRenderDelegate::DestroyRprim(HdRprim* rPrim)
{
  delete rPrim;
}

HdSprim*
HdBridgeRenderDelegate::CreateSprim(TfToken const& typeId,
                                    SdfPath const& sprimId)
{
  if (typeId == HdPrimTypeTokens->camera) {
    return new HdCamera(sprimId);
  }
  if (typeId == HdPrimTypeTokens->material) {
    // return new HdMaterial(sprimId);
  }
  if (typeId == HdPrimTypeTokens->distantLight) {
    return new HdBridgeDistantLight(sprimId, _sender);
  }
  if (typeId == HdPrimTypeTokens->sphereLight) {
    return new HdBridgeSphereLight(sprimId, _sender);
  }

  TF_CODING_ERROR(
    "Unknown Sprim type=%s id=%s", typeId.GetText(), sprimId.GetText());
  return nullptr;
}

HdSprim*
HdBridgeRenderDelegate::CreateFallbackSprim(TfToken const& typeId)
{
  if (typeId == HdPrimTypeTokens->camera) {
    return new HdCamera(SdfPath::EmptyPath());
  }
  if (typeId == HdPrimTypeTokens->material) {
    // return new HdMaterial(SdfPath::EmptyPath());
  }
  if (typeId == HdPrimTypeTokens->distantLight) {
    return new HdBridgeDistantLight(SdfPath::EmptyPath(), _sender);
  }
  if (typeId == HdPrimTypeTokens->sphereLight) {
    return new HdBridgeSphereLight(SdfPath::EmptyPath(), _sender);
  }

  TF_CODING_ERROR("Creating unknown fallback sprim type=%s", typeId.GetText());
  return nullptr;
}

void
HdBridgeRenderDelegate::DestroySprim(HdSprim* sPrim)
{
  delete sPrim;
}

HdBprim*
HdBridgeRenderDelegate::CreateBprim(TfToken const& typeId,
                                    SdfPath const& bprimId)
{
  TF_CODING_ERROR(
    "Unknown Bprim type=%s id=%s", typeId.GetText(), bprimId.GetText());
  return nullptr;
}

HdBprim*
HdBridgeRenderDelegate::CreateFallbackBprim(TfToken const& typeId)
{
  TF_CODING_ERROR("Creating unknown fallback bprim type=%s", typeId.GetText());
  return nullptr;
}

void
HdBridgeRenderDelegate::DestroyBprim(HdBprim* bPrim)
{
  delete bPrim;
}

HdInstancer*
HdBridgeRenderDelegate::CreateInstancer(HdSceneDelegate* delegate,
                                        SdfPath const& id)
{
  TF_CODING_ERROR("Creating Instancer not supported id=%s", id.GetText());
  return nullptr;
}

void
HdBridgeRenderDelegate::DestroyInstancer(HdInstancer* instancer)
{
  TF_CODING_ERROR("Destroy instancer not supported");
}

HdRenderParam*
HdBridgeRenderDelegate::GetRenderParam() const
{
  return nullptr;
}
