#ifndef SYNC_TASK_H
#define SYNC_TASK_H

#include "bridgeSender.h"
#include "pxr/imaging/hd/renderPass.h"
#include "pxr/pxr.h"
#include "pxr/usdImaging/usdImaging/delegate.h"

using namespace pxr;

class BridgeRenderPass : public HdRenderPass
{
public:
  BridgeRenderPass(HdRenderIndex* index, HdRprimCollection const& collection)
    : HdRenderPass(index, collection)
  {
  }
  virtual ~BridgeRenderPass() {}

  void _Execute(HdRenderPassStateSharedPtr const& renderPassState,
                TfTokenVector const& renderTags) override
  {
  }
};

class SyncTask final : public HdTask
{
public:
  SyncTask(HdRenderPassSharedPtr const& renderPass,
           TfTokenVector const& renderTags,
           rust::Box<BridgeSendEndNotifier> notifier)
    : HdTask(SdfPath::EmptyPath())
    , _renderPass(renderPass)
    , _renderTags(renderTags)
    , _notifier(std::move(notifier))
  {
  }

  virtual void Sync(HdSceneDelegate* delegate,
                    HdTaskContext* ctx,
                    HdDirtyBits* dirtyBits) override;

  virtual void Prepare(HdTaskContext* ctx, HdRenderIndex* renderIndex) override;

  virtual void Execute(HdTaskContext* ctx) override;

  virtual const TfTokenVector& GetRenderTags() const override;

private:
  HdRenderPassSharedPtr _renderPass;
  TfTokenVector _renderTags;
  rust::Box<BridgeSendEndNotifier> _notifier;
};

#endif
