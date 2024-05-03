#ifndef BRIDGE_SCENE_INDEX_OBSERVER_H
#define BRIDGE_SCENE_INDEX_OBSERVER_H

#include "cameraObserver.h"
#include "distantLightObserver.h"
#include "materialObserver.h"
#include "meshObserver.h"
#include "pxr/imaging/hd/dataSource.h"
#include "pxr/imaging/hd/sceneIndexObserver.h"
#include "pxr/pxr.h"
#include "renderSettingsObserver.h"
#include "sphereLightObserver.h"
#include "usdDataDiff.h"
#include <iostream>

using namespace pxr;

class HdBridgeSceneIndexObserver final : public HdSceneIndexObserver
{

public:
  HdBridgeSceneIndexObserver();
  ~HdBridgeSceneIndexObserver() override;

  void PrimsAdded(const HdSceneIndexBase& sender,
                  const AddedPrimEntries& entries) override;

  void PrimsRemoved(const HdSceneIndexBase& sender,
                    const RemovedPrimEntries& entries) override;

  void PrimsDirtied(const HdSceneIndexBase& sender,
                    const DirtiedPrimEntries& entries) override;

  void PrimsRenamed(const HdSceneIndexBase& sender,
                    const RenamedPrimEntries& entries) override;

  void ClearDiff();

  void GetDiff(const HdSceneIndexBase& sender, UsdDataDiff& diff);

private:
  RenderSettingsObserver _renderSettingsObserver;
  MeshObserver _meshObserver;
  SphereLightObserver _sphereLightObserver;
  DistantLightObserver _distantLightObserver;
  CameraObserver _cameraObserver;
  MaterialObserver _materialObserver;

  // This class does not support copying.
  HdBridgeSceneIndexObserver(const HdBridgeSceneIndexObserver&) = delete;
  HdBridgeSceneIndexObserver& operator=(const HdBridgeSceneIndexObserver&) =
    delete;
};

#endif
