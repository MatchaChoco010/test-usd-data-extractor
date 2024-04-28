#include "usdDataExtractor.h"
#include "usd_data_extractor/src/bridge.rs.h"

BridgeUsdDataExtractor::BridgeUsdDataExtractor(rust::Box<BridgeSender> sender,
                                               std::string openPath)
  : _sender(std::make_shared<rust::Box<BridgeSender>>(std::move(sender)))
  , _openPath(openPath)
  , _engine()
  , _stage()
  , _renderDelegate(_sender)
  , _renderIndex(nullptr)
  , _delegate(nullptr)
  , _renderSettingsPath(SdfPath::EmptyPath())
  , _renderProductPath(SdfPath::EmptyPath())
{
  _renderIndex = HdRenderIndex::New(&_renderDelegate, HdDriverVector());
  _delegate = new UsdImagingDelegate(_renderIndex, SdfPath::AbsoluteRootPath());

  _stage = UsdStage::Open(_openPath);
  if (!_stage) {
    throw std::runtime_error("Failed to open stage");
  }

  _delegate->Populate(_stage->GetPseudoRoot());

  HdRprimCollection collection = HdRprimCollection(
    HdTokens->geometry, HdReprSelector(HdReprTokens->refined));
  _renderPass =
    HdRenderPassSharedPtr(new BridgeRenderPass(_renderIndex, collection));

  TfTokenVector renderTags;
  renderTags.push_back(HdRenderTagTokens->geometry);
  _renderTags = renderTags;

  double startTimeCode = _stage->GetStartTimeCode();
  double endTimeCode = _stage->GetEndTimeCode();
  (*_sender)->time_code_range(startTimeCode, endTimeCode);
}

BridgeUsdDataExtractor::~BridgeUsdDataExtractor()
{
  delete _delegate;
  delete _renderIndex;
}

void
BridgeUsdDataExtractor::extract(rust::Box<BridgeSendEndNotifier> notifier,
                                double timeCode)
{
  _delegate->SetTime(timeCode);

  HdTaskSharedPtrVector tasks = {
    std::make_shared<SyncTask>(
      _renderPass, TfTokenVector(), std::move(notifier)),
  };
  _engine.Execute(&_delegate->GetRenderIndex(), &tasks);
}

rust::Vec<rust::String>
BridgeUsdDataExtractor::get_render_settings_paths()
{
  rust::Vec<rust::String> result;
  for (const UsdPrim& prim : _stage->GetPseudoRoot().GetDescendants()) {
    if (prim.IsA<UsdRenderSettings>()) {
      std::string path = prim.GetPath().GetString();
      result.push_back(rust::string(path));
    }
  }
  return result;
}

void
BridgeUsdDataExtractor::set_render_settings_path(rust::Str p)
{
  SdfPath path = SdfPath(std::string(p));
  for (const UsdPrim& prim : _stage->GetPseudoRoot().GetDescendants()) {
    if (prim.IsA<UsdRenderSettings>()) {
      if (prim.GetPath() == path) {
        _renderSettingsPath = path;
        return;
      }
    }
  }
  throw std::runtime_error("RenderSettings path not found");
}

void
BridgeUsdDataExtractor::clear_render_settings_path()
{
  _renderSettingsPath = SdfPath::EmptyPath();
}

rust::Vec<rust::String>
BridgeUsdDataExtractor::get_render_product_paths()
{
  if (_renderSettingsPath.IsEmpty()) {
    throw std::runtime_error("RenderSettings path is empty");
  }

  UsdRenderSettings renderSettings(_stage->GetPrimAtPath(_renderSettingsPath));
  if (!renderSettings) {
    throw std::runtime_error("RenderSettings not found");
  }

  UsdRelationship productsRel = renderSettings.GetProductsRel();
  SdfPathVector productPaths;
  productsRel.GetTargets(&productPaths);

  rust::Vec<rust::String> result;
  for (const SdfPath& productPath : productPaths) {
    result.push_back(rust::string(productPath.GetString()));
  }
  return result;
}

void
BridgeUsdDataExtractor::set_render_product_path(rust::Str p)
{
  SdfPath path = SdfPath(std::string(p));
  for (const UsdPrim& prim : _stage->GetPseudoRoot().GetDescendants()) {
    if (prim.IsA<UsdRenderProduct>()) {
      if (prim.GetPath() == path) {
        _renderProductPath = path;
        return;
      }
    }
  }
  throw std::runtime_error("RenderProduct path not found");
}

void
BridgeUsdDataExtractor::clear_render_product_path()
{
  _renderProductPath = SdfPath::EmptyPath();
}

rust::String
BridgeUsdDataExtractor::get_active_camera_path()
{
  if (_renderProductPath.IsEmpty()) {
    throw std::runtime_error("RenderProduct path is empty");
  }

  UsdRenderProduct renderProduct(_stage->GetPrimAtPath(_renderProductPath));
  if (!renderProduct) {
    throw std::runtime_error("RenderProduct not found");
  }

  UsdRelationship cameraRel = renderProduct.GetCameraRel();
  SdfPathVector cameraPaths;
  cameraRel.GetTargets(&cameraPaths);
  for (const SdfPath& cameraPath : cameraPaths) {
    return rust::string(cameraPath.GetString());
  }

  throw std::runtime_error("Camera path not found");
}

std::unique_ptr<BridgeUsdDataExtractor>
new_usd_data_extractor(rust::Box<BridgeSender> sender, rust::Str openPath)
{
  return std::make_unique<BridgeUsdDataExtractor>(std::move(sender),
                                                  std::string(openPath));
}
