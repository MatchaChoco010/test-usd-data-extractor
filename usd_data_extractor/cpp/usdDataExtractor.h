#ifndef BRIDGE_USD_DATA_EXTRACTOR_H
#define BRIDGE_USD_DATA_EXTRACTOR_H

#include <iostream>
#include <memory>

#include "bridgeSender.h"
#include "pxr/imaging/hd/changeTracker.h"
#include "pxr/imaging/hd/engine.h"
#include "pxr/imaging/hd/renderIndex.h"
#include "pxr/imaging/hd/renderPass.h"
#include "pxr/imaging/hd/rprim.h"
#include "pxr/imaging/hd/rprimCollection.h"
#include "pxr/imaging/hd/tokens.h"
#include "pxr/pxr.h"
#include "pxr/usd/sdf/path.h"
#include "pxr/usd/usdRender/product.h"
#include "pxr/usd/usdRender/settings.h"
#include "pxr/usdImaging/usdImaging/delegate.h"
#include "renderDelegate.h"
#include "rust/cxx.h"
#include "syncTask.h"

using namespace pxr;

class BridgeUsdDataExtractor
{
public:
  BridgeUsdDataExtractor(rust::Box<BridgeSender> sender, std::string openPath);
  virtual ~BridgeUsdDataExtractor();

  void extract(rust::Box<BridgeSendEndNotifier> notifier, double timeCode);
  rust::Vec<rust::String> get_render_settings_paths();
  void set_render_settings_path(rust::Str path);
  void clear_render_settings_path();
  rust::Vec<rust::String> get_render_product_paths();
  void set_render_product_path(rust::Str path);
  void clear_render_product_path();
  rust::String get_active_camera_path();

private:
  BridgeSenderSharedPtr _sender;
  std::string _openPath;
  HdEngine _engine;
  UsdStageRefPtr _stage;
  HdBridgeRenderDelegate _renderDelegate;
  HdRenderIndex* _renderIndex;
  UsdImagingDelegate* _delegate;
  HdRenderPassSharedPtr _renderPass;
  TfTokenVector _renderTags;
  SdfPath _renderSettingsPath;
  SdfPath _renderProductPath;
};

std::unique_ptr<BridgeUsdDataExtractor>
new_usd_data_extractor(rust::Box<BridgeSender> sender, rust::Str openPath);

#endif
