#pragma once
#include <memory>
#include <iostream>
#include "rust/cxx.h"

class BridgeUsdDataExtractor
{
public:
  BridgeUsdDataExtractor(std::string openPath);
  ~BridgeUsdDataExtractor();

  // void extractData(const std::string& usdFilePath);
};

std::unique_ptr<BridgeUsdDataExtractor> new_usd_data_extractor(rust::Str openPath);
