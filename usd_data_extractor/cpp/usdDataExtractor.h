#pragma once
#include <memory>
#include <iostream>

class BridgeUsdDataExtractor
{
public:
  BridgeUsdDataExtractor();
  ~BridgeUsdDataExtractor();

  // void extractData(const std::string& usdFilePath);
};

std::unique_ptr<BridgeUsdDataExtractor> new_usd_data_extractor();
