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
  _meshObserver.PrimsAdded(sender, entries);
  _sphereLightObserver.PrimsAdded(sender, entries);
}

void
HdBridgeSceneIndexObserver::PrimsRemoved(const HdSceneIndexBase& sender,
                                         const RemovedPrimEntries& entries)
{
  _meshObserver.PrimsRemoved(sender, entries);
  _sphereLightObserver.PrimsRemoved(sender, entries);
}

void
HdBridgeSceneIndexObserver::PrimsDirtied(const HdSceneIndexBase& sender,
                                         const DirtiedPrimEntries& entries)
{
  _meshObserver.PrimsDirtied(sender, entries);
  _sphereLightObserver.PrimsDirtied(sender, entries);
}

void
HdBridgeSceneIndexObserver::PrimsRenamed(const HdSceneIndexBase& sender,
                                         const RenamedPrimEntries& entries)
{
  _meshObserver.PrimsRenamed(sender, entries);
  _sphereLightObserver.PrimsRenamed(sender, entries);
}

void
HdBridgeSceneIndexObserver::ClearDiff()
{
  _meshObserver.ClearDiff();
  _sphereLightObserver.ClearDiff();
}

void
HdBridgeSceneIndexObserver::GetDiff(const HdSceneIndexBase& sender,
                                    UsdDataDiff& diff)
{
  _meshObserver.GetDiff(sender, diff);
  _sphereLightObserver.GetDiff(sender, diff);
}
