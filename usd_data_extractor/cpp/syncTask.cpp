#include "syncTask.h"
#include "usd_data_extractor/src/bridge.rs.h"

void
SyncTask::Sync(HdSceneDelegate* sceneDelegate,
               HdTaskContext* ctx,
               HdDirtyBits* dirtyBits)

{
  _renderPass->Sync();
  *dirtyBits = HdChangeTracker::Clean;
}

void
SyncTask::Prepare(HdTaskContext* ctx, HdRenderIndex* renderIndex)
{
}

void
SyncTask::Execute(HdTaskContext* ctx)
{
  _notifier->notify();
}

const TfTokenVector&
SyncTask::GetRenderTags() const
{
  return _renderTags;
}
