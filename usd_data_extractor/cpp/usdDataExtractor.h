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

  void extract(rust::Box<BridgeSendEndNotifier> notifier);

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
};

std::unique_ptr<BridgeUsdDataExtractor>
new_usd_data_extractor(rust::Box<BridgeSender> sender, rust::Str openPath);

#endif
