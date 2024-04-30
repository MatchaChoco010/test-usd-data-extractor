#ifndef BRIDGE_USD_DATA_EXTRACTOR_H
#define BRIDGE_USD_DATA_EXTRACTOR_H

#include "pxr/imaging/hd/tokens.h"
#include "pxr/pxr.h"
#include "pxr/usd/sdf/path.h"
#include "pxr/usd/usd/stage.h"
#include "pxr/usdImaging/usdImaging/sceneIndices.h"
#include "pxr/usdImaging/usdImaging/stageSceneIndex.h"
#include "rust/cxx.h"
#include "sceneIndexObserver.h"
#include "usdDataDiff.h"
#include <iostream>
#include <memory>

using namespace pxr;

class BridgeUsdDataExtractor
{
public:
  BridgeUsdDataExtractor(std::string openPath);
  virtual ~BridgeUsdDataExtractor();

  double start_time_code() const { return _startTimeCode; }
  double end_time_code() const { return _endTimeCode; }

  void extract(double timeCode, UsdDataDiff& diff);

private:
  std::string _openPath;
  UsdStageRefPtr _stage;
  double _startTimeCode;
  double _endTimeCode;

  bool _isFirstExtract = true;

  HdBridgeSceneIndexObserver _observer;
  UsdImagingStageSceneIndexRefPtr _stageSceneIndex;
  HdSceneIndexBaseRefPtr _sceneIndex;
};

std::unique_ptr<BridgeUsdDataExtractor>
new_usd_data_extractor(rust::Str openPath);

#endif
