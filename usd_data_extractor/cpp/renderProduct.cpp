#include "renderProduct.h"
#include "usd_data_extractor/src/bridge.rs.h"

PXR_NAMESPACE_OPEN_SCOPE

TF_DEFINE_PUBLIC_TOKENS(HdBridgeRenderProductTokens, (renderProduct));

PXR_NAMESPACE_CLOSE_SCOPE

HdBridgeRenderProduct::HdBridgeRenderProduct(SdfPath const& id,
                                             BridgeSenderSharedPtr sender)
  : HdSprim(id)
  , _id(id)
  , _sender(sender)
{
  rust::String path = rust::string(this->_id.GetText());
  (*_sender)->create_render_product(path);
}

HdBridgeRenderProduct::~HdBridgeRenderProduct()
{
  rust::String path = rust::string(this->_id.GetText());
  (*_sender)->destroy_render_product(path);
}

HdDirtyBits
HdBridgeRenderProduct::GetInitialDirtyBitsMask() const
{
  HdDirtyBits mask = HdChangeTracker::Clean;
  return mask;
}

void
HdBridgeRenderProduct::Sync(HdSceneDelegate* sceneDelegate,
                            HdRenderParam* renderParam,
                            HdDirtyBits* dirtyBits)
{
  _SyncCameraPath(sceneDelegate);
  *dirtyBits = HdChangeTracker::Clean;
}

void
HdBridgeRenderProduct::_SyncCameraPath(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());
  rust::Box<RenderProductData> renderProductData = new_render_product_data();

  const VtValue cameraValue = sceneDelegate->Get(_id, TfToken("camera"));
  if (cameraValue.IsHolding<SdfPath>()) {
    SdfPath cameraPath = cameraValue.UncheckedGet<SdfPath>();
    renderProductData->set_camera_path(rust::string(cameraPath.GetText()));
  }

  (*_sender)->render_product_data(path, std::move(renderProductData));
}
