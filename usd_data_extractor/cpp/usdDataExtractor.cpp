#include "usdDataExtractor.h"

BridgeUsdDataExtractor::BridgeUsdDataExtractor()
{
  // Constructor
  std::cout << "BridgeUsdDataExtractor constructor called" << std::endl;
}

BridgeUsdDataExtractor::~BridgeUsdDataExtractor()
{
  // Destructor
}

std::unique_ptr<BridgeUsdDataExtractor> new_usd_data_extractor()
{
  return std::make_unique<BridgeUsdDataExtractor>();
}
