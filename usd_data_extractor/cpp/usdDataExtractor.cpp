#include "usdDataExtractor.h"

BridgeUsdDataExtractor::BridgeUsdDataExtractor(std::string openPath)
{
  // Constructor
  std::cout << "BridgeUsdDataExtractor constructor called" << std::endl;
  std::cout << "=> open path=\"" << openPath << "\"" << std::endl;
}

BridgeUsdDataExtractor::~BridgeUsdDataExtractor()
{
  // Destructor
}

std::unique_ptr<BridgeUsdDataExtractor> new_usd_data_extractor(rust::Str openPath)
{
  return std::make_unique<BridgeUsdDataExtractor>(std::string(openPath));
}
