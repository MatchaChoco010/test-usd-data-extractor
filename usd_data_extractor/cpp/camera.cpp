#include "camera.h"
#include "usd_data_extractor/src/bridge.rs.h"

HdBridgeCamera::HdBridgeCamera(SdfPath const& id, BridgeSenderSharedPtr sender)
  : HdCamera(id)
  , _id(id)
  , _sender(sender)
{
  rust::String path = rust::string(this->_id.GetText());
  (*_sender)->create_camera(path);
}

HdBridgeCamera::~HdBridgeCamera()
{
  rust::String path = rust::string(this->_id.GetText());
  (*_sender)->destroy_camera(path);
}

HdDirtyBits
HdBridgeCamera::GetInitialDirtyBitsMask() const
{
  HdDirtyBits mask =
    HdCamera::Clean | HdCamera::DirtyParams | HdCamera::DirtyTransform;
  // | HdChangeTracker::DirtyVisibility | HdChangeTracker::DirtyInstancer;
  return mask;
}

void
HdBridgeCamera::Sync(HdSceneDelegate* sceneDelegate,
                     HdRenderParam* renderParam,
                     HdDirtyBits* dirtyBits)
{
  if (*dirtyBits & HdCamera::DirtyParams) {
    _SyncCameraData(sceneDelegate);
  }

  if (*dirtyBits & HdCamera::DirtyTransform) {
    _SyncTransform(sceneDelegate);
  }

  *dirtyBits = HdCamera::Clean;
}

void
HdBridgeCamera::_SyncTransform(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());

  GfMatrix4d matrix = sceneDelegate->GetTransform(_id);
  const double* data = matrix.GetArray();
  rust::Slice<const double> dataSlice{ data, 16 };

  (*_sender)->transform_matrix(path, dataSlice);
}

void
HdBridgeCamera::_SyncCameraData(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());
  rust::Box<CameraData> cameraData = new_camera_data();

  VtValue focalLengthValue =
    sceneDelegate->GetCameraParamValue(_id, HdCameraTokens->focalLength);
  if (!focalLengthValue.IsEmpty() && focalLengthValue.IsHolding<float>()) {
    cameraData->set_focal_length(focalLengthValue.Get<float>());
  }

  VtValue verticalApertureValue =
    sceneDelegate->GetCameraParamValue(_id, HdCameraTokens->verticalAperture);
  if (!verticalApertureValue.IsEmpty() &&
      verticalApertureValue.IsHolding<float>()) {
    cameraData->set_vertical_aperture(verticalApertureValue.Get<float>());
  }

  (*_sender)->camera_data(path, std::move(cameraData));
}
