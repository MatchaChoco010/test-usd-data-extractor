#ifndef BRIDGE_USD_DATA_EXTRACTOR_H
#define BRIDGE_USD_DATA_EXTRACTOR_H

#include <memory>
#include <iostream>
#include "rust/cxx.h"

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
};

std::unique_ptr<BridgeUsdDataExtractor> new_usd_data_extractor(rust::Box<BridgeSender> sender, rust::Str openPath);

#endif
