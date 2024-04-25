#ifndef BRIDGE_USD_DATA_EXTRACTOR_H
#define BRIDGE_USD_DATA_EXTRACTOR_H

#include <memory>
#include <iostream>
#include "rust/cxx.h"

#include "pxr/pxr.h"
#include "pxr/usdImaging/usdImaging/delegate.h"
// #include "pxr/imaging/hd/changeTracker.h"
#include "pxr/imaging/hd/engine.h"
// #include "pxr/imaging/hd/renderIndex.h"
// #include "pxr/imaging/hd/renderPass.h"
// #include "pxr/imaging/hd/rprim.h"
// #include "pxr/imaging/hd/rprimCollection.h"
// #include "pxr/imaging/hd/tokens.h"

struct BridgeSender;
struct BridgeSendEndNotifier;

typedef std::shared_ptr<rust::Box<BridgeSender>> BridgeSenderSharedPtr;

class BridgeUsdDataExtractor
{
public:
  BridgeUsdDataExtractor(rust::Box<BridgeSender> sender, std::string openPath);
  ~BridgeUsdDataExtractor();

  void extract(rust::Box<BridgeSendEndNotifier> notifier) const;

private:
  BridgeSenderSharedPtr _sender;
  std::string _openPath;
  pxr::HdEngine _engine;
  pxr::UsdStageRefPtr _stage;
};

std::unique_ptr<BridgeUsdDataExtractor> new_usd_data_extractor(rust::Box<BridgeSender> sender, rust::Str openPath);

#endif
