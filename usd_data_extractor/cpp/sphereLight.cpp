#include "sphereLight.h"
#include "usd_data_extractor/src/bridge.rs.h"

HdBridgeSphereLight::HdBridgeSphereLight(SdfPath const& id,
                                         BridgeSenderSharedPtr sender)
  : HdLight(id)
  , _id(id)
  , _sender(sender)
{
  rust::String path = rust::string(this->_id.GetText());
  (*_sender)->create_sphere_light(path);
}

HdBridgeSphereLight::~HdBridgeSphereLight()
{
  rust::String path = rust::string(this->_id.GetText());
  (*_sender)->destroy_sphere_light(path);
}

HdDirtyBits
HdBridgeSphereLight::GetInitialDirtyBitsMask() const
{
  HdDirtyBits mask =
    HdLight::Clean | HdLight::DirtyParams | HdLight::DirtyTransform;
  // | HdChangeTracker::DirtyVisibility | HdChangeTracker::DirtyInstancer;
  return mask;
}

void
HdBridgeSphereLight::Sync(HdSceneDelegate* sceneDelegate,
                          HdRenderParam* renderParam,
                          HdDirtyBits* dirtyBits)
{
  if (*dirtyBits & HdLight::DirtyParams) {
    _SyncSphereLightData(sceneDelegate);
  }

  if (*dirtyBits & HdLight::DirtyTransform) {
    _SyncTransform(sceneDelegate);
  }

  *dirtyBits = HdChangeTracker::Clean;
}

void
HdBridgeSphereLight::_SyncTransform(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());

  GfMatrix4d matrix = sceneDelegate->GetTransform(_id);
  const double* data = matrix.GetArray();
  rust::Slice<const double> dataSlice{ data, 16 };

  (*_sender)->transform_matrix(path, dataSlice);
}

void
HdBridgeSphereLight::_SyncSphereLightData(HdSceneDelegate* sceneDelegate)
{
  rust::String path = rust::string(this->_id.GetText());
  rust::Box<SphereLightData> sphereLightData = new_sphere_light_data();

  VtValue colorValue =
    sceneDelegate->GetLightParamValue(_id, HdLightTokens->color);
  if (colorValue.IsHolding<GfVec3f>()) {
    GfVec3f color = colorValue.Get<GfVec3f>();
    sphereLightData->set_color(color[0], color[1], color[2]);
  }

  VtValue intensityValue =
    sceneDelegate->GetLightParamValue(_id, HdLightTokens->intensity);
  if (intensityValue.IsHolding<float>()) {
    float intensity = intensityValue.Get<float>();
    sphereLightData->set_intensity(intensity);
  }

  VtValue radiusValue =
    sceneDelegate->GetLightParamValue(_id, HdLightTokens->radius);
  if (radiusValue.IsHolding<float>()) {
    float radius = radiusValue.Get<float>();
    sphereLightData->set_radius(radius);
  }

  VtValue angleValue =
    sceneDelegate->GetLightParamValue(_id, HdLightTokens->shapingConeAngle);
  if (angleValue.IsHolding<float>()) {
    float angle = angleValue.Get<float>();
    sphereLightData->set_cone_angle(angle);
  }

  VtValue softnessValue =
    sceneDelegate->GetLightParamValue(_id, HdLightTokens->shapingConeSoftness);
  if (softnessValue.IsHolding<float>()) {
    float softness = softnessValue.Get<float>();
    sphereLightData->set_cone_softness(softness);
  }

  (*_sender)->sphere_light_data(path, std::move(sphereLightData));
}
