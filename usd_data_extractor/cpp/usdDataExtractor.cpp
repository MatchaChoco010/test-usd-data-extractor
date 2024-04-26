#include "usdDataExtractor.h"
#include "usd_data_extractor/src/bridge.rs.h"

BridgeUsdDataExtractor::BridgeUsdDataExtractor(rust::Box<BridgeSender> sender,
                                               std::string openPath)
  : _sender(std::make_shared<rust::Box<BridgeSender>>(std::move(sender)))
  , _openPath(openPath)
  , _engine()
  , _stage()
  , _renderDelegate(_sender)
  , _renderIndex(nullptr)
  , _delegate(nullptr)
{
  _renderIndex = HdRenderIndex::New(&_renderDelegate, HdDriverVector());
  _delegate = new UsdImagingDelegate(_renderIndex, SdfPath::AbsoluteRootPath());

  _stage = UsdStage::Open(_openPath);

  _delegate->Populate(_stage->GetPseudoRoot());

  HdRprimCollection collection = HdRprimCollection(
    HdTokens->geometry, HdReprSelector(HdReprTokens->refined));
  _renderPass =
    HdRenderPassSharedPtr(new BridgeRenderPass(_renderIndex, collection));

  TfTokenVector renderTags;
  renderTags.push_back(HdRenderTagTokens->geometry);
  _renderTags = renderTags;
}

BridgeUsdDataExtractor::~BridgeUsdDataExtractor()
{
  delete _delegate;
  delete _renderIndex;
}

void
BridgeUsdDataExtractor::extract(rust::Box<BridgeSendEndNotifier> notifier)
{
  double startTimeCode = _stage->GetStartTimeCode();
  double endTimeCode = _stage->GetEndTimeCode();
  (*_sender)->send_string(
    rust::String("=> start time code=" + std::to_string(startTimeCode)));
  (*_sender)->send_string(
    rust::String("=> end time code=" + std::to_string(endTimeCode)));

  HdTaskSharedPtrVector tasks = {
    std::make_shared<SyncTask>(
      _renderPass, TfTokenVector(), std::move(notifier)),
  };
  _engine.Execute(&_delegate->GetRenderIndex(), &tasks);
}

std::unique_ptr<BridgeUsdDataExtractor>
new_usd_data_extractor(rust::Box<BridgeSender> sender, rust::Str openPath)
{
  return std::make_unique<BridgeUsdDataExtractor>(std::move(sender),
                                                  std::string(openPath));
}
