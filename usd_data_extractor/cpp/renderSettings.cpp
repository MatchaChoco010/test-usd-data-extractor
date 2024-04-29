#include "renderSettings.h"
#include "usd_data_extractor/src/bridge.rs.h"

HdBridgeRenderSettings::HdBridgeRenderSettings(SdfPath const& id,
                                               BridgeSenderSharedPtr sender)
  : HdRenderSettings(id)
  , _id(id)
  , _sender(sender)
{
  rust::String path = rust::string(this->_id.GetText());
  (*_sender)->create_render_settings(path);
}

HdBridgeRenderSettings::~HdBridgeRenderSettings()
{
  rust::String path = rust::string(this->_id.GetText());
  (*_sender)->destroy_render_settings(path);
}

HdDirtyBits
HdBridgeRenderSettings::GetInitialDirtyBitsMask() const
{
  HdDirtyBits mask =
    HdRenderSettings::Clean | HdRenderSettings::DirtyRenderProducts;
  return mask;
}

void
HdBridgeRenderSettings::_Sync(HdSceneDelegate* sceneDelegate,
                              HdRenderParam* renderParam,
                              const HdDirtyBits* dirtyBits)
{
  if (*dirtyBits & HdRenderSettings::DirtyRenderProducts) {
    _SyncRenderProducts(sceneDelegate);
  }
}

void
HdBridgeRenderSettings::_SyncRenderProducts(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());
  rust::Box<RenderSettingsData> renderSettingsData = new_render_settings_data();

  const VtValue vProducts =
    sceneDelegate->Get(_id, HdRenderSettingsPrimTokens->renderProducts);
  if (vProducts.IsHolding<RenderProducts>()) {
    RenderProducts products = vProducts.UncheckedGet<RenderProducts>();
    std::vector<rust::String> productPaths;
    for (const auto& product : products) {
      productPaths.push_back(rust::String(product.productPath.GetText()));
    }
    rust::Slice<const rust::String> productPathsSlice{ productPaths.data(),
                                                       productPaths.size() };
    renderSettingsData->set_render_product_paths(productPathsSlice);
  }

  (*_sender)->render_settings_data(path, std::move(renderSettingsData));
}
