#include "usdDataExtractor.h"
#include "usd_data_extractor/src/bridge.rs.h"

BridgeUsdDataExtractor::BridgeUsdDataExtractor(rust::Box<BridgeSender> sender, std::string openPath)
    : _sender(std::make_shared<rust::Box<BridgeSender>>(std::move(sender))),
      _openPath(openPath)
{
  // Constructor
  std::cout << "BridgeUsdDataExtractor constructor called" << std::endl;
}

BridgeUsdDataExtractor::~BridgeUsdDataExtractor()
{
  // Destructor
}

void BridgeUsdDataExtractor::extract(rust::Box<BridgeSendEndNotifier> notifier) const
{
  // Extract USD data
  (*_sender)->send_string(rust::String("extract called from C++!"));
  (*_sender)->send_string(rust::String("=> open path=\"" + _openPath + "\""));
  notifier->notify();
}

std::unique_ptr<BridgeUsdDataExtractor> new_usd_data_extractor(rust::Box<BridgeSender> sender, rust::Str openPath)
{
  return std::make_unique<BridgeUsdDataExtractor>(std::move(sender), std::string(openPath));
}
