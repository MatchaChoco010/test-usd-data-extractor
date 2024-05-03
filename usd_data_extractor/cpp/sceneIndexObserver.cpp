#include "sceneIndexObserver.h"
#include "usd_data_extractor/src/bridge.rs.h"

HdBridgeSceneIndexObserver::HdBridgeSceneIndexObserver()
  : HdSceneIndexObserver()
{
}

HdBridgeSceneIndexObserver::~HdBridgeSceneIndexObserver() {}

void
HdBridgeSceneIndexObserver::PrimsAdded(const HdSceneIndexBase& sender,
                                       const AddedPrimEntries& entries)
{
  _renderSettingsObserver.PrimsAdded(sender, entries);
  _meshObserver.PrimsAdded(sender, entries);
  _sphereLightObserver.PrimsAdded(sender, entries);
  _distantLightObserver.PrimsAdded(sender, entries);
  _cameraObserver.PrimsAdded(sender, entries);
  _materialObserver.PrimsAdded(sender, entries);
}

void
HdBridgeSceneIndexObserver::PrimsRemoved(const HdSceneIndexBase& sender,
                                         const RemovedPrimEntries& entries)
{
  _renderSettingsObserver.PrimsRemoved(sender, entries);
  _meshObserver.PrimsRemoved(sender, entries);
  _sphereLightObserver.PrimsRemoved(sender, entries);
  _distantLightObserver.PrimsRemoved(sender, entries);
  _cameraObserver.PrimsRemoved(sender, entries);
  _materialObserver.PrimsRemoved(sender, entries);
}

void
HdBridgeSceneIndexObserver::PrimsDirtied(const HdSceneIndexBase& sender,
                                         const DirtiedPrimEntries& entries)
{
  _renderSettingsObserver.PrimsDirtied(sender, entries);
  _meshObserver.PrimsDirtied(sender, entries);
  _sphereLightObserver.PrimsDirtied(sender, entries);
  _distantLightObserver.PrimsDirtied(sender, entries);
  _cameraObserver.PrimsDirtied(sender, entries);
  _materialObserver.PrimsDirtied(sender, entries);
}

void
HdBridgeSceneIndexObserver::PrimsRenamed(const HdSceneIndexBase& sender,
                                         const RenamedPrimEntries& entries)
{
  _renderSettingsObserver.PrimsRenamed(sender, entries);
  _meshObserver.PrimsRenamed(sender, entries);
  _sphereLightObserver.PrimsRenamed(sender, entries);
  _distantLightObserver.PrimsRenamed(sender, entries);
  _cameraObserver.PrimsRenamed(sender, entries);
  _materialObserver.PrimsRenamed(sender, entries);
}

void
HdBridgeSceneIndexObserver::ClearDiff()
{
  _renderSettingsObserver.ClearDiff();
  _meshObserver.ClearDiff();
  _sphereLightObserver.ClearDiff();
  _distantLightObserver.ClearDiff();
  _cameraObserver.ClearDiff();
  _materialObserver.ClearDiff();
}

void
HdBridgeSceneIndexObserver::GetDiff(const HdSceneIndexBase& sender,
                                    UsdDataDiff& diff)
{
  _renderSettingsObserver.GetDiff(sender, diff);
  _meshObserver.GetDiff(sender, diff);
  _sphereLightObserver.GetDiff(sender, diff);
  _distantLightObserver.GetDiff(sender, diff);
  _cameraObserver.GetDiff(sender, diff);
  _materialObserver.GetDiff(sender, diff);
}
