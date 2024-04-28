#include "distantLight.h"
#include "usd_data_extractor/src/bridge.rs.h"

HdBridgeDistantLight::HdBridgeDistantLight(SdfPath const& id,
                                           BridgeSenderSharedPtr sender)
  : HdLight(id)
  , _id(id)
  , _sender(sender)
{
  rust::String path = rust::string(this->_id.GetText());
  (*_sender)->create_distant_light(path);
}

HdBridgeDistantLight::~HdBridgeDistantLight()
{
  rust::String path = rust::string(this->_id.GetText());
  (*_sender)->destroy_distant_light(path);
}

HdDirtyBits
HdBridgeDistantLight::GetInitialDirtyBitsMask() const
{
  HdDirtyBits mask =
    HdLight::Clean | HdLight::DirtyParams | HdLight::DirtyTransform;
  // | HdChangeTracker::DirtyVisibility | HdChangeTracker::DirtyInstancer;
  return mask;
}

void
HdBridgeDistantLight::Sync(HdSceneDelegate* sceneDelegate,
                           HdRenderParam* renderParam,
                           HdDirtyBits* dirtyBits)
{
  if (*dirtyBits & HdLight::DirtyParams) {
    _SyncDistantLightData(sceneDelegate);
  }

  if (*dirtyBits & HdLight::DirtyTransform) {
    _SyncTransform(sceneDelegate);
  }

  *dirtyBits = HdLight::Clean;
}

void
HdBridgeDistantLight::_SyncTransform(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());

  GfMatrix4d matrix = sceneDelegate->GetTransform(_id);
  const double* data = matrix.GetArray();
  rust::Slice<const double> dataSlice{ data, 16 };

  (*_sender)->transform_matrix(path, dataSlice);
}

void
HdBridgeDistantLight::_SyncDistantLightData(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());
  rust::Box<DistantLightData> distantLightData = new_distant_light_data();

  VtValue colorValue =
    sceneDelegate->GetLightParamValue(_id, HdLightTokens->color);
  if (colorValue.IsHolding<GfVec3f>()) {
    GfVec3f color = colorValue.Get<GfVec3f>();
    distantLightData->set_color(color[0], color[1], color[2]);
  }

  VtValue intensityValue =
    sceneDelegate->GetLightParamValue(_id, HdLightTokens->intensity);
  if (intensityValue.IsHolding<float>()) {
    float intensity = intensityValue.Get<float>();
    distantLightData->set_intensity(intensity);
  }

  VtValue angleValue =
    sceneDelegate->GetLightParamValue(_id, HdLightTokens->angle);
  if (angleValue.IsHolding<float>()) {
    float angle = angleValue.Get<float>();
    distantLightData->set_angle(angle);
  }

  (*_sender)->distant_light_data(path, std::move(distantLightData));
}
