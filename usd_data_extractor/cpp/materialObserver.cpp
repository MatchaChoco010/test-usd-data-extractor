#include "materialObserver.h"
#include "usd_data_extractor/src/bridge.rs.h"

MaterialObserver::MaterialObserver() {}

MaterialObserver::~MaterialObserver() {}

void
MaterialObserver::PrimsAdded(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::AddedPrimEntries& entries)
{
  for (const auto entry : entries) {
    auto primType = entry.primType;

    if (primType != TypeToken) {
      continue;
    }

    // stageに追加されたMaterialを記録する
    _materialPaths.insert(entry.primPath);

    if (_removed.find(entry.primPath) != _removed.end()) {
      // このDiff中ですでにremovedされているDiffがある場合、
      // removedを取り消してaddedとして扱う
      _removed.erase(entry.primPath);
      _added.emplace(entry.primPath);
    } else if (_dirtied.find(entry.primPath) != _dirtied.end()) {
      // このDiff中ですでにdirtiedされているDiffがある場合、
      // dirtiedを取り消してaddedとして扱う
      _dirtied.erase(entry.primPath);
      _added.emplace(entry.primPath);
    } else {
      // _addedされたMaterialとしてdiffに登録する
      _added.emplace(entry.primPath);
    }
  }
}

void
MaterialObserver::PrimsRemoved(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::RemovedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // _materialPathに記録されていない場合は無視する
    if (_materialPaths.find(entry.primPath) == _materialPaths.end()) {
      continue;
    }

    // stageから削除されたMaterialを記録から削除する
    _materialPaths.erase(entry.primPath);

    if (_added.find(entry.primPath) != _added.end()) {
      // このDiff中ですでにaddedされているDiffがある場合、
      // addedを取り消して差分はなかったことにする
      _added.erase(entry.primPath);
    } else if (_dirtied.find(entry.primPath) != _dirtied.end()) {
      // このDiff中ですでにdirtiedされているDiffがある場合、
      // そのdirtiedは削除されるので取り消してremovedだけを記録する
      _dirtied.erase(entry.primPath);
      _removed.emplace(entry.primPath);
    } else {
      // _removedされたMaterialとしてdiffに登録する
      _removed.emplace(entry.primPath);
    }
  }
}

void
MaterialObserver::PrimsDirtied(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::DirtiedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // _materialPathに記録されていない場合は無視する
    if (_materialPaths.find(entry.primPath) == _materialPaths.end()) {
      continue;
    }

    // このフレーム中でaddedな場合は、addedですべての情報を送るので追加で差分を送る必要はない
    // そのため、addedされたMaterialの場合はdirtiedを無視する
    if (_added.find(entry.primPath) != _added.end()) {
      continue;
    }

    // dirtiedされたらdiffに記録する
    _dirtied.emplace(entry.primPath);
  }
}

void
MaterialObserver::PrimsRenamed(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::RenamedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // MaterialPathに記録されていない場合は無視する
    if (_materialPaths.find(entry.oldPrimPath) == _materialPaths.end()) {
      continue;
    }

    // stageからrenameされたMaterialを記録から削除し、新しい名前で記録する
    _materialPaths.erase(entry.oldPrimPath);
    _materialPaths.insert(entry.newPrimPath);

    // oldPathをremoveする
    {
      if (_added.find(entry.oldPrimPath) != _added.end()) {
        // このDiff中ですでにaddedされているDiffがある場合、
        // addedを取り消して差分はなかったことにする
        _added.erase(entry.oldPrimPath);
      } else if (_dirtied.find(entry.oldPrimPath) != _dirtied.end()) {
        // このDiff中ですでにdirtiedされているDiffがある場合、
        // そのdirtiedは削除されるので取り消す
        _dirtied.erase(entry.oldPrimPath);
        _removed.emplace(entry.oldPrimPath);
      } else {
        // _removedされたMaterialとしてdiffに登録する
        _removed.emplace(entry.oldPrimPath);
      }
    }

    // newPathをaddする
    {
      if (_removed.find(entry.newPrimPath) != _removed.end()) {
        // このDiff中ですでにremovedされているDiffがある場合、
        // removedを取り消してaddedとして扱う
        _removed.erase(entry.newPrimPath);
        _added.emplace(entry.newPrimPath);
      } else if (_dirtied.find(entry.newPrimPath) != _dirtied.end()) {
        // このDiff中ですでにdirtiedされているDiffがある場合、
        // dirtiedを取り消してaddedとして扱う
        _dirtied.erase(entry.newPrimPath);
        _added.emplace(entry.newPrimPath);
      } else {
        // _addedされたMaterialとしてdiffに登録する
        _added.emplace(entry.newPrimPath);
      }
    }
  }
}

void
MaterialObserver::ClearDiff()
{
  // 各種diffの記録をクリアする
  _added.clear();
  _removed.clear();
  _dirtied.clear();
}

std::optional<rust::String>
MaterialObserver::_GetMaterialFilePath(
  const HdSceneIndexBase& sceneIndex,
  const SdfPath& path,
  const HdDataSourceLocator connectionLocator) const
{
  auto connectionSource = sceneIndex.GetDataSource(path, connectionLocator);
  if (connectionSource) {
    auto vectorConnectionSource = HdVectorDataSource::Cast(connectionSource);

    if (vectorConnectionSource->GetNumElements() == 0) {
      return std::nullopt;
    }

    auto elementSource = vectorConnectionSource->GetElement(0);
    auto containerSource = HdContainerDataSource::Cast(elementSource);

    auto connectionSource = containerSource->Get(UpstreamNodePathToken);
    auto sampledConnectionSource = HdSampledDataSource::Cast(connectionSource);

    auto value = sampledConnectionSource->GetValue(0);
    auto nodePath = value.Get<TfToken>();

    auto fileLocator =
      NodesLocator.Append(nodePath).Append(FileParameterLocator);
    auto fileSource = sceneIndex.GetDataSource(path, fileLocator);
    if (fileSource) {
      auto sampledFileSource = HdSampledDataSource::Cast(fileSource);
      auto value = sampledFileSource->GetValue(0);
      auto assetPath = value.Get<SdfAssetPath>();
      auto data = rust::String(assetPath.GetResolvedPath());
      return data;
    }
  }
  return std::nullopt;
}

void
MaterialObserver::_UpdateDiff(const HdSceneIndexBase& sceneIndex,
                              UsdDataDiff& diff,
                              const SdfPath path) const
{
  // terminal nodeがUsdPreviewSurfaceでない場合は無視する
  auto terminalNodeSource =
    sceneIndex.GetDataSource(path, TerminalNodePathLocator);
  if (!terminalNodeSource) {
    return;
  }
  auto sampledTerminalNodeSource =
    HdSampledDataSource::Cast(terminalNodeSource);
  auto value = sampledTerminalNodeSource->GetValue(0);
  auto terminalNode = value.Get<TfToken>();

  auto terminalNodeIdentifierLocator =
    NodesLocator.Append(terminalNode).Append(NodeIdentifierLocator);
  auto terminalNodeIdentifierSource =
    sceneIndex.GetDataSource(path, terminalNodeIdentifierLocator);
  if (!terminalNodeIdentifierSource) {
    return;
  }
  auto sampledTerminalNodeIdentifierSource =
    HdSampledDataSource::Cast(terminalNodeIdentifierSource);
  auto terminalNodeIdentifierValue =
    sampledTerminalNodeIdentifierSource->GetValue(0);
  auto terminalNodeIdentifier = terminalNodeIdentifierValue.Get<TfToken>();
  if (terminalNodeIdentifier != UsdPreviewSurfaceToken) {
    return;
  }

  // Materialのdiffを作成する
  auto pathString = rust::String(path.GetText());
  diff.add_or_update_material(pathString);

  // diffuseColorのパラメーター情報をdiffに登録する
  auto diffuseColorParameterLocator =
    NodesLocator.Append(terminalNode).Append(DiffuseColorParameterLocator);
  auto diffuseColorParameterSource =
    sceneIndex.GetDataSource(path, diffuseColorParameterLocator);
  if (diffuseColorParameterSource) {
    auto sampledDiffuseColorParameterSource =
      HdSampledDataSource::Cast(diffuseColorParameterSource);
    auto value = sampledDiffuseColorParameterSource->GetValue(0);
    auto data = value.Get<GfVec3f>();
    diff.add_or_update_material_diffuse_color(
      pathString, data[0], data[1], data[2]);
  }

  // emissiveのパラメーター情報をdiffに登録する
  auto emissiveParameterLocator =
    NodesLocator.Append(terminalNode).Append(EmissiveParameterLocator);
  auto emissiveParameterSource =
    sceneIndex.GetDataSource(path, emissiveParameterLocator);
  if (emissiveParameterSource) {
    auto sampledEmissiveParameterSource =
      HdSampledDataSource::Cast(emissiveParameterSource);
    auto value = sampledEmissiveParameterSource->GetValue(0);
    auto data = value.Get<GfVec3f>();
    diff.add_or_update_material_emissive(pathString, data[0], data[1], data[2]);
  }

  // metallicのパラメーター情報をdiffに登録する
  auto metallicParameterLocator =
    NodesLocator.Append(terminalNode).Append(MetallicParameterLocator);
  auto metallicParameterSource =
    sceneIndex.GetDataSource(path, metallicParameterLocator);
  if (metallicParameterSource) {
    auto sampledMetallicParameterSource =
      HdSampledDataSource::Cast(metallicParameterSource);
    auto value = sampledMetallicParameterSource->GetValue(0);
    auto data = value.Get<float>();
    diff.add_or_update_material_metallic(pathString, data);
  }

  // opacityのパラメーター情報をdiffに登録する
  auto opacityParameterLocator =
    NodesLocator.Append(terminalNode).Append(OpacityParameterLocator);
  auto opacityParameterSource =
    sceneIndex.GetDataSource(path, opacityParameterLocator);
  if (opacityParameterSource) {
    auto sampledOpacityParameterSource =
      HdSampledDataSource::Cast(opacityParameterSource);
    auto value = sampledOpacityParameterSource->GetValue(0);
    auto data = value.Get<float>();
    diff.add_or_update_material_opacity(pathString, data);
  }

  // roughnessのパラメーター情報をdiffに登録する
  auto roughnessParameterLocator =
    NodesLocator.Append(terminalNode).Append(RoughnessParameterLocator);
  auto roughnessParameterSource =
    sceneIndex.GetDataSource(path, roughnessParameterLocator);
  if (roughnessParameterSource) {
    auto sampledRoughnessParameterSource =
      HdSampledDataSource::Cast(roughnessParameterSource);
    auto value = sampledRoughnessParameterSource->GetValue(0);
    auto data = value.Get<float>();
    diff.add_or_update_material_roughness(pathString, data);
  }

  // diffuseColorのコネクション情報をdiffに登録する
  auto diffuseColorConnectionLocator =
    NodesLocator.Append(terminalNode).Append(DiffuseColorConnectionLocator);
  auto diffuseColorFilePath =
    _GetMaterialFilePath(sceneIndex, path, diffuseColorConnectionLocator);
  if (diffuseColorFilePath) {
    diff.add_or_update_material_diffuse_color_file(
      pathString, diffuseColorFilePath.value());
  }

  // emissiveのコネクション情報をdiffに登録する
  auto emissiveConnectionLocator =
    NodesLocator.Append(terminalNode).Append(EmissiveConnectionLocator);
  auto emissiveFilePath =
    _GetMaterialFilePath(sceneIndex, path, emissiveConnectionLocator);
  if (emissiveFilePath) {
    diff.add_or_update_material_emissive_file(pathString,
                                              emissiveFilePath.value());
  }

  // metallicのコネクション情報をdiffに登録する
  auto metallicConnectionLocator =
    NodesLocator.Append(terminalNode).Append(MetallicConnectionLocator);
  auto metallicFilePath =
    _GetMaterialFilePath(sceneIndex, path, metallicConnectionLocator);
  if (metallicFilePath) {
    diff.add_or_update_material_metallic_file(pathString,
                                              metallicFilePath.value());
  }

  // normalのコネクション情報をdiffに登録する
  auto normalConnectionLocator =
    NodesLocator.Append(terminalNode).Append(NormalConnectionLocator);
  auto normalFilePath =
    _GetMaterialFilePath(sceneIndex, path, normalConnectionLocator);
  if (normalFilePath) {
    diff.add_or_update_material_normal_file(pathString, normalFilePath.value());
  }

  // opacityのコネクション情報をdiffに登録する
  auto opacityConnectionLocator =
    NodesLocator.Append(terminalNode).Append(OpacityConnectionLocator);
  auto opacityFilePath =
    _GetMaterialFilePath(sceneIndex, path, opacityConnectionLocator);
  if (opacityFilePath) {
    diff.add_or_update_material_opacity_file(pathString,
                                             opacityFilePath.value());
  }

  // roughnessのコネクション情報をdiffに登録する
  auto roughnessConnectionLocator =
    NodesLocator.Append(terminalNode).Append(RoughnessConnectionLocator);
  auto roughnessFilePath =
    _GetMaterialFilePath(sceneIndex, path, roughnessConnectionLocator);
  if (roughnessFilePath) {
    diff.add_or_update_material_roughness_file(pathString,
                                               roughnessFilePath.value());
  }
}

void
MaterialObserver::GetDiff(const HdSceneIndexBase& sceneIndex, UsdDataDiff& diff)
{
  // addedされたMaterialの情報をdiffに登録する
  for (const auto& path : _added) {
    _UpdateDiff(sceneIndex, diff, path);
  }

  // removedされたMaterialの情報をdiffに登録する
  for (const auto& path : _removed) {
    auto pathString = rust::String(path.GetText());
    diff.destroy_material(pathString);
  }

  // dirtiedされたMaterialの情報をdiffに登録する
  for (const auto& path : _dirtied) {
    _UpdateDiff(sceneIndex, diff, path);
  }
}
