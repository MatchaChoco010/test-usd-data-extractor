#include "usdDataExtractor.h"
#include "usd_data_extractor/src/bridge.rs.h"

BridgeUsdDataExtractor::BridgeUsdDataExtractor(rust::Box<BridgeSender> sender, std::string openPath)
    : _sender(std::make_shared<rust::Box<BridgeSender>>(std::move(sender))),
      _openPath(openPath),
      _engine(),
      _stage()
{
  _stage = pxr::UsdStage::Open(_openPath);
}

BridgeUsdDataExtractor::~BridgeUsdDataExtractor()
{
}

void BridgeUsdDataExtractor::extract(rust::Box<BridgeSendEndNotifier> notifier) const
{
  // Extract USD data
  (*_sender)->send_string(rust::String("extract data!"));

  double startTimeCode = _stage->GetStartTimeCode();
  double endTimeCode = _stage->GetEndTimeCode();
  (*_sender)->send_string(rust::String("=> start time code=" + std::to_string(startTimeCode)));
  (*_sender)->send_string(rust::String("=> end time code=" + std::to_string(endTimeCode)));

  // Traverse the stage
  for (pxr::UsdPrim prim: _stage->Traverse())
  {
    std::string path = prim.GetPath().GetAsString();
    (*_sender)->send_string(rust::String("=> prim path=\"" + path + "\""));
  }

  notifier->notify();
}

std::unique_ptr<BridgeUsdDataExtractor> new_usd_data_extractor(rust::Box<BridgeSender> sender, rust::Str openPath)
{
  return std::make_unique<BridgeUsdDataExtractor>(std::move(sender), std::string(openPath));
}
