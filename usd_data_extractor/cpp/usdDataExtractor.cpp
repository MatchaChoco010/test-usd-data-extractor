#include "usdDataExtractor.h"
#include "usd_data_extractor/src/bridge.rs.h"

BridgeUsdDataExtractor::BridgeUsdDataExtractor(std::string openPath)
  : _openPath(openPath)
{
  _stage = UsdStage::Open(_openPath);
  if (!_stage) {
    throw std::runtime_error("Failed to open stage");
  }

  _startTimeCode = _stage->GetStartTimeCode();
  _endTimeCode = _stage->GetEndTimeCode();

  UsdImagingCreateSceneIndicesInfo info;
  const UsdImagingSceneIndices sceneIndices =
    UsdImagingCreateSceneIndices(info);

  _stageSceneIndex = sceneIndices.stageSceneIndex;
  _sceneIndex = sceneIndices.finalSceneIndex;

  _sceneIndex->AddObserver(HdSceneIndexObserverPtr(&_observer));

  _stageSceneIndex->SetStage(_stage);
}

BridgeUsdDataExtractor::~BridgeUsdDataExtractor()
{
  if (_sceneIndex) {
    _sceneIndex->RemoveObserver(HdSceneIndexObserverPtr(&_observer));
  }
}

void
BridgeUsdDataExtractor::extract(double timeCode, UsdDataDiff& diff)
{
  if (!_isFirstExtract) {
    _observer.ClearDiff();
  }

  // debug
  // HdUtils::PrintSceneIndex(std::cout, _sceneIndex);

  _stageSceneIndex->SetTime(timeCode);
  _observer.GetDiff(*_sceneIndex, diff);

  if (_isFirstExtract) {
    _isFirstExtract = false;
  }
}

std::unique_ptr<BridgeUsdDataExtractor>
new_usd_data_extractor(rust::Str openPath)
{
  return std::make_unique<BridgeUsdDataExtractor>(std::string(openPath));
}
