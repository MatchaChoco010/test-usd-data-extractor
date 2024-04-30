#include "meshObserver.h"
#include "usd_data_extractor/src/bridge.rs.h"

MeshObserver::MeshObserver() {}

MeshObserver::~MeshObserver() {}

void
MeshObserver::PrimsAdded(const HdSceneIndexBase& sender,
                         const HdSceneIndexObserver::AddedPrimEntries& entries)
{
  for (const auto entry : entries) {
    auto primType = entry.primType;

    if (primType != TypeToken) {
      continue;
    }

    // stageに追加されたMeshを記録する
    _meshPaths.insert(entry.primPath);

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
      // _addedされたMeshとしてdiffに登録する
      _added.emplace(entry.primPath);
    }
  }
}

void
MeshObserver::PrimsRemoved(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::RemovedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // meshPathに記録されていない場合は無視する
    if (_meshPaths.find(entry.primPath) == _meshPaths.end()) {
      continue;
    }

    // stageから削除されたMeshを記録から削除する
    _meshPaths.erase(entry.primPath);

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
      // _removedされたMeshとしてdiffに登録する
      _removed.emplace(entry.primPath);
    }
  }
}

void
MeshObserver::PrimsDirtied(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::DirtiedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // meshPathに記録されていない場合は無視する
    if (_meshPaths.find(entry.primPath) == _meshPaths.end()) {
      continue;
    }

    // このフレーム中でaddedな場合は、addedですべての情報を送るので追加で差分を送る必要はない
    // そのため、addedされたMeshの場合はdirtiedを無視する
    if (_added.find(entry.primPath) != _added.end()) {
      continue;
    }

    // dirtiedされたMeshのlocatorによって、どのDiffを登録するかを決定する
    for (const auto locator : entry.dirtyLocators) {
      if (locator.HasPrefix(TransforLocator)) {
        // xformについて差分がある場合、transformのmatrixを再取得する
        _dirtied[entry.primPath].insert(DiffType::TransformMatrix);
      } else if (locator.HasPrefix(PrimvarsLocator) ||
                 locator.HasPrefix(MaterialBindingsLocator) ||
                 locator.HasPrefix(MeshLocator)) {
        // primvars, materialBindings, meshのいずれかについて差分がある場合、
        // meshの全データを再取得する
        _dirtied[entry.primPath].insert(DiffType::MeshData);
      }
    }
  }
}

void
MeshObserver::PrimsRenamed(
  const HdSceneIndexBase& sender,
  const HdSceneIndexObserver::RenamedPrimEntries& entries)
{
  for (const auto entry : entries) {
    // meshPathに記録されていない場合は無視する
    if (_meshPaths.find(entry.oldPrimPath) == _meshPaths.end()) {
      continue;
    }

    // stageからrenameされたMeshを記録から削除し、新しい名前で記録する
    _meshPaths.erase(entry.oldPrimPath);
    _meshPaths.insert(entry.newPrimPath);

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
        // _removedされたMeshとしてdiffに登録する
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
        // _addedされたMeshとしてdiffに登録する
        _added.emplace(entry.newPrimPath);
      }
    }
  }
}

void
MeshObserver::ClearDiff()
{
  // 各種diffの記録をクリアする
  _added.clear();
  _removed.clear();
  _dirtied.clear();
}

void
MeshObserver::GetDiff(const HdSceneIndexBase& sceneIndex, UsdDataDiff& diff)
{
  // addedされたMeshの情報をdiffに登録する
  for (const auto& path : _added) {
    auto pathString = rust::String(path.GetText());

    diff.create_mesh(pathString);

    auto transformMatrixSource =
      sceneIndex.GetDataSource(path, TransformMatrixDataLocator);
    if (transformMatrixSource) {
      auto sampledTransformMatrixSource =
        HdSampledDataSource::Cast(transformMatrixSource);
      auto value = sampledTransformMatrixSource->GetValue(0);
      auto matrix = value.Get<GfMatrix4d>();
      auto matrixArray = matrix.GetArray();
      std::array<float, 16> matrixData;
      for (int i = 0; i < 16; i++) {
        matrixData[i] = matrixArray[i];
      }
      auto data = rust::Slice<const float>(matrixData.data(), 16);
      diff.create_mesh_transform_matrix(pathString, data);
    }

    auto leftHandedSource =
      sceneIndex.GetDataSource(path, LeftHandedDataLocator);
    if (leftHandedSource) {
      auto sampledLeftHandedSource =
        HdSampledDataSource::Cast(leftHandedSource);
      auto value = sampledLeftHandedSource->GetValue(0);
      auto orientation = value.Get<TfToken>();
      if (orientation == TfToken("leftHanded")) {
        diff.create_mesh_left_handed(pathString, true);
      }
    }

    auto pointsSource = sceneIndex.GetDataSource(path, PointsDataLocator);
    if (pointsSource) {
      auto sampledPointsSource = HdSampledDataSource::Cast(pointsSource);
      auto value = sampledPointsSource->GetValue(0);
      auto points = value.Get<VtVec3fArray>();
      auto data = reinterpret_cast<const float*>(points.cdata());
      auto size = points.size() * 3;
      auto pointsData = rust::Slice<const float>(data, size);
      diff.create_mesh_points(pathString, pointsData);
    }

    auto normalsSource = sceneIndex.GetDataSource(path, NormalsDataLocator);
    if (normalsSource) {
      auto sampledNormalsSource = HdSampledDataSource::Cast(normalsSource);
      auto value = sampledNormalsSource->GetValue(0);
      auto normals = value.Get<VtVec3fArray>();
      auto data = reinterpret_cast<const float*>(normals.cdata());
      auto size = normals.size() * 3;
      auto normalsData = rust::Slice<const float>(data, size);
      diff.create_mesh_normals(pathString, normalsData);
    }

    auto normalsInterpolationSource =
      sceneIndex.GetDataSource(path, NormalsInterpolationDataLocator);
    if (normalsInterpolationSource) {
      auto sampledNormalsInterpolationSource =
        HdSampledDataSource::Cast(normalsInterpolationSource);
      auto value = sampledNormalsInterpolationSource->GetValue(0);
      auto interpolation = value.Get<TfToken>();
      if (interpolation == TfToken("constant")) {
        diff.create_mesh_normals_interpolation(pathString,
                                               Interpolation::Constant);
      } else if (interpolation == TfToken("uniform")) {
        diff.create_mesh_normals_interpolation(pathString,
                                               Interpolation::Uniform);
      } else if (interpolation == TfToken("varying")) {
        diff.create_mesh_normals_interpolation(pathString,
                                               Interpolation::Varying);
      } else if (interpolation == TfToken("vertex")) {
        diff.create_mesh_normals_interpolation(pathString,
                                               Interpolation::Vertex);
      } else if (interpolation == TfToken("faceVarying")) {
        diff.create_mesh_normals_interpolation(pathString,
                                               Interpolation::FaceVarying);
      } else if (interpolation == TfToken("instance")) {
        diff.create_mesh_normals_interpolation(pathString,
                                               Interpolation::Instance);
      }
    }

    auto uvsSource = sceneIndex.GetDataSource(path, UVsDataLocator);
    if (uvsSource) {
      auto sampledUVsSource = HdSampledDataSource::Cast(uvsSource);
      auto value = sampledUVsSource->GetValue(0);
      auto uvs = value.Get<VtVec2fArray>();
      auto data = reinterpret_cast<const float*>(uvs.cdata());
      auto size = uvs.size() * 2;
      auto uvsData = rust::Slice<const float>(data, size);
      diff.create_mesh_uvs(pathString, uvsData);
    }

    auto uvsInterpolationSource =
      sceneIndex.GetDataSource(path, UVsInterpolationDataLocator);
    if (uvsInterpolationSource) {
      auto sampledUVsInterpolationSource =
        HdSampledDataSource::Cast(uvsInterpolationSource);
      auto value = sampledUVsInterpolationSource->GetValue(0);
      auto interpolation = value.Get<TfToken>();
      if (interpolation == TfToken("constant")) {
        diff.create_mesh_uvs_interpolation(pathString, Interpolation::Constant);
      } else if (interpolation == TfToken("uniform")) {
        diff.create_mesh_uvs_interpolation(pathString, Interpolation::Uniform);
      } else if (interpolation == TfToken("varying")) {
        diff.create_mesh_uvs_interpolation(pathString, Interpolation::Varying);
      } else if (interpolation == TfToken("vertex")) {
        diff.create_mesh_uvs_interpolation(pathString, Interpolation::Vertex);
      } else if (interpolation == TfToken("faceVarying")) {
        diff.create_mesh_uvs_interpolation(pathString,
                                           Interpolation::FaceVarying);
      } else if (interpolation == TfToken("instance")) {
        diff.create_mesh_uvs_interpolation(pathString, Interpolation::Instance);
      }
    }

    auto faceVertexIndicesSource =
      sceneIndex.GetDataSource(path, FaceVertexIndicesLocator);
    if (faceVertexIndicesSource) {
      auto sampledFaceVertexIndicesSource =
        HdSampledDataSource::Cast(faceVertexIndicesSource);
      auto value = sampledFaceVertexIndicesSource->GetValue(0);
      auto faceVertexIndices = value.Get<VtIntArray>();
      auto size = faceVertexIndices.size();
      std::vector<uint32_t> data;
      data.reserve(size);
      for (int i = 0; i < size; i++) {
        data.push_back(faceVertexIndices[i]);
      }
      auto faceVertexIndicesData =
        rust::Slice<const uint32_t>(data.data(), size);
      diff.create_mesh_face_vertex_indices(pathString, faceVertexIndicesData);
    }

    auto faceVertexCountsSource =
      sceneIndex.GetDataSource(path, FaceVertexCountsLocator);
    if (faceVertexCountsSource) {
      auto sampledFaceVertexCountsSource =
        HdSampledDataSource::Cast(faceVertexCountsSource);
      auto value = sampledFaceVertexCountsSource->GetValue(0);
      auto faceVertexCounts = value.Get<VtIntArray>();
      auto size = faceVertexCounts.size();
      std::vector<uint32_t> data;
      data.reserve(size);
      for (int i = 0; i < size; i++) {
        data.push_back(faceVertexCounts[i]);
      }
      auto faceVertexCountsData =
        rust::Slice<const uint32_t>(data.data(), size);
      diff.create_mesh_face_vertex_counts(pathString, faceVertexCountsData);
    }

    auto geomSubsetSource = sceneIndex.GetDataSource(path, GeomSubsetLocator);
    if (geomSubsetSource) {
      auto containerGeomSubsetSource =
        HdContainerDataSource::Cast(geomSubsetSource);

      for (const auto& name : containerGeomSubsetSource->GetNames()) {
        auto typeLocator = HdDataSourceLocator(
          TfToken("mesh"), TfToken("geomSubsets"), name, TfToken("type"));
        auto typeSource = sceneIndex.GetDataSource(path, typeLocator);
        auto sampledTypeSource = HdSampledDataSource::Cast(typeSource);
        auto typeValue = sampledTypeSource->GetValue(0);
        auto typeData = typeValue.Get<TfToken>();
        auto ty = rust::String(typeData.GetString());

        auto indicesLocator = HdDataSourceLocator(
          TfToken("mesh"), TfToken("geomSubsets"), name, TfToken("indices"));
        auto indicesSource = sceneIndex.GetDataSource(path, indicesLocator);
        auto sampledIndicesSource = HdSampledDataSource::Cast(indicesSource);
        auto indicesValue = sampledIndicesSource->GetValue(0);
        auto indicesData = indicesValue.Get<VtIntArray>();
        auto size = indicesData.size();
        std::vector<uint32_t> data;
        data.reserve(size);
        for (int i = 0; i < size; i++) {
          data.push_back(indicesData[i]);
        }
        auto indices = rust::Slice<const uint32_t>(data.data(), size);

        auto nameString = rust::String(name.GetString());

        diff.create_mesh_geom_subset(pathString, nameString, ty, indices);
      }
    }
  }

  // removedされたMeshの情報をdiffに登録する
  for (const auto& path : _removed) {
    auto pathString = rust::String(path.GetText());
    diff.destroy_mesh(pathString);
  }

  // dirtiedされたMeshの情報をdiffに登録する
  for (const auto& it : _dirtied) {
    auto path = it.first;
    auto diffTypes = it.second;

    auto pathString = rust::String(path.GetText());

    for (const auto& diffType : diffTypes) {

      if (diffType == DiffType::TransformMatrix) {
        // transformのmatrixを再取得する
        auto transformMatrixSource =
          sceneIndex.GetDataSource(path, TransformMatrixDataLocator);
        if (transformMatrixSource) {
          auto sampledTransformMatrixSource =
            HdSampledDataSource::Cast(transformMatrixSource);
          auto value = sampledTransformMatrixSource->GetValue(0);
          auto matrix = value.Get<GfMatrix4d>();
          auto matrixArray = matrix.GetArray();
          std::array<float, 16> matrixData;
          for (int i = 0; i < 16; i++) {
            matrixData[i] = matrixArray[i];
          }
          auto data = rust::Slice<const float>(matrixData.data(), 16);
          diff.diff_mesh_transform_matrix(pathString, data);
        }
      } else if (diffType == DiffType::MeshData) {
        // 頂点属性等のデータに差分があるので、Meshの一通りのデータを再取得する
        diff.diff_mesh_data(pathString);

        auto leftHandedSource =
          sceneIndex.GetDataSource(path, LeftHandedDataLocator);
        if (leftHandedSource) {
          auto sampledLeftHandedSource =
            HdSampledDataSource::Cast(leftHandedSource);
          auto value = sampledLeftHandedSource->GetValue(0);
          auto orientation = value.Get<TfToken>();
          if (orientation == TfToken("leftHanded")) {
            diff.diff_mesh_data_left_handed(pathString, true);
          }
        }

        auto pointsSource = sceneIndex.GetDataSource(path, PointsDataLocator);
        if (pointsSource) {
          auto sampledPointsSource = HdSampledDataSource::Cast(pointsSource);
          auto value = sampledPointsSource->GetValue(0);
          auto points = value.Get<VtVec3fArray>();
          auto data = reinterpret_cast<const float*>(points.cdata());
          auto size = points.size() * 3;
          auto pointsData = rust::Slice<const float>(data, size);
          diff.diff_mesh_data_points(pathString, pointsData);
        }

        auto normalsSource = sceneIndex.GetDataSource(path, NormalsDataLocator);
        if (normalsSource) {
          auto sampledNormalsSource = HdSampledDataSource::Cast(normalsSource);
          auto value = sampledNormalsSource->GetValue(0);
          auto normals = value.Get<VtVec3fArray>();
          auto data = reinterpret_cast<const float*>(normals.cdata());
          auto size = normals.size() * 3;
          auto normalsData = rust::Slice<const float>(data, size);
          diff.diff_mesh_data_normals(pathString, normalsData);
        }

        auto normalsInterpolationSource =
          sceneIndex.GetDataSource(path, NormalsInterpolationDataLocator);
        if (normalsInterpolationSource) {
          auto sampledNormalsInterpolationSource =
            HdSampledDataSource::Cast(normalsInterpolationSource);
          auto value = sampledNormalsInterpolationSource->GetValue(0);
          auto interpolation = value.Get<TfToken>();
          if (interpolation == TfToken("constant")) {
            diff.diff_mesh_data_normals_interpolation(pathString,
                                                      Interpolation::Constant);
          } else if (interpolation == TfToken("uniform")) {
            diff.diff_mesh_data_normals_interpolation(pathString,
                                                      Interpolation::Uniform);
          } else if (interpolation == TfToken("varying")) {
            diff.diff_mesh_data_normals_interpolation(pathString,
                                                      Interpolation::Varying);
          } else if (interpolation == TfToken("vertex")) {
            diff.diff_mesh_data_normals_interpolation(pathString,
                                                      Interpolation::Vertex);
          } else if (interpolation == TfToken("faceVarying")) {
            diff.diff_mesh_data_normals_interpolation(
              pathString, Interpolation::FaceVarying);
          } else if (interpolation == TfToken("instance")) {
            diff.diff_mesh_data_normals_interpolation(pathString,
                                                      Interpolation::Instance);
          }
        }

        auto uvsSource = sceneIndex.GetDataSource(path, UVsDataLocator);
        if (uvsSource) {
          auto sampledUVsSource = HdSampledDataSource::Cast(uvsSource);
          auto value = sampledUVsSource->GetValue(0);
          auto uvs = value.Get<VtVec2fArray>();
          auto data = reinterpret_cast<const float*>(uvs.cdata());
          auto size = uvs.size() * 2;
          auto uvsData = rust::Slice<const float>(data, size);
          diff.diff_mesh_data_uvs(pathString, uvsData);
        }

        auto uvsInterpolationSource =
          sceneIndex.GetDataSource(path, UVsInterpolationDataLocator);
        if (uvsInterpolationSource) {
          auto sampledUVsInterpolationSource =
            HdSampledDataSource::Cast(uvsInterpolationSource);
          auto value = sampledUVsInterpolationSource->GetValue(0);
          auto interpolation = value.Get<TfToken>();
          if (interpolation == TfToken("constant")) {
            diff.diff_mesh_data_uvs_interpolation(pathString,
                                                  Interpolation::Constant);
          } else if (interpolation == TfToken("uniform")) {
            diff.diff_mesh_data_uvs_interpolation(pathString,
                                                  Interpolation::Uniform);
          } else if (interpolation == TfToken("varying")) {
            diff.diff_mesh_data_uvs_interpolation(pathString,
                                                  Interpolation::Varying);
          } else if (interpolation == TfToken("vertex")) {
            diff.diff_mesh_data_uvs_interpolation(pathString,
                                                  Interpolation::Vertex);
          } else if (interpolation == TfToken("faceVarying")) {
            diff.diff_mesh_data_uvs_interpolation(pathString,
                                                  Interpolation::FaceVarying);
          } else if (interpolation == TfToken("instance")) {
            diff.diff_mesh_data_uvs_interpolation(pathString,
                                                  Interpolation::Instance);
          }
        }

        auto faceVertexIndicesSource =
          sceneIndex.GetDataSource(path, FaceVertexIndicesLocator);
        if (faceVertexIndicesSource) {
          auto sampledFaceVertexIndicesSource =
            HdSampledDataSource::Cast(faceVertexIndicesSource);
          auto value = sampledFaceVertexIndicesSource->GetValue(0);
          auto faceVertexIndices = value.Get<VtIntArray>();
          auto size = faceVertexIndices.size();
          std::vector<uint32_t> data;
          data.reserve(size);
          for (int i = 0; i < size; i++) {
            data.push_back(faceVertexIndices[i]);
          }
          auto faceVertexIndicesData =
            rust::Slice<const uint32_t>(data.data(), size);
          diff.diff_mesh_data_face_vertex_indices(pathString,
                                                  faceVertexIndicesData);
        }

        auto faceVertexCountsSource =
          sceneIndex.GetDataSource(path, FaceVertexCountsLocator);
        if (faceVertexCountsSource) {
          auto sampledFaceVertexCountsSource =
            HdSampledDataSource::Cast(faceVertexCountsSource);
          auto value = sampledFaceVertexCountsSource->GetValue(0);
          auto faceVertexCounts = value.Get<VtIntArray>();
          auto size = faceVertexCounts.size();
          std::vector<uint32_t> data;
          data.reserve(size);
          for (int i = 0; i < size; i++) {
            data.push_back(faceVertexCounts[i]);
          }
          auto faceVertexCountsData =
            rust::Slice<const uint32_t>(data.data(), size);
          diff.diff_mesh_data_face_vertex_counts(pathString,
                                                 faceVertexCountsData);
        }

        auto geomSubsetSource =
          sceneIndex.GetDataSource(path, GeomSubsetLocator);
        if (geomSubsetSource) {
          auto containerGeomSubsetSource =
            HdContainerDataSource::Cast(geomSubsetSource);

          for (const auto& name : containerGeomSubsetSource->GetNames()) {
            auto typeLocator = HdDataSourceLocator(
              TfToken("mesh"), TfToken("geomSubsets"), name, TfToken("type"));
            auto typeSource = sceneIndex.GetDataSource(path, typeLocator);
            auto sampledTypeSource = HdSampledDataSource::Cast(typeSource);
            auto typeValue = sampledTypeSource->GetValue(0);
            auto typeData = typeValue.Get<TfToken>();
            auto ty = rust::String(typeData.GetString());

            auto indicesLocator = HdDataSourceLocator(TfToken("mesh"),
                                                      TfToken("geomSubsets"),
                                                      name,
                                                      TfToken("indices"));
            auto indicesSource = sceneIndex.GetDataSource(path, indicesLocator);
            auto sampledIndicesSource =
              HdSampledDataSource::Cast(indicesSource);
            auto indicesValue = sampledIndicesSource->GetValue(0);
            auto indicesData = indicesValue.Get<VtIntArray>();
            auto size = indicesData.size();
            std::vector<uint32_t> data;
            data.reserve(size);
            for (int i = 0; i < size; i++) {
              data.push_back(indicesData[i]);
            }
            auto indices = rust::Slice<const uint32_t>(data.data(), size);

            auto nameString = rust::String(name.GetString());

            diff.diff_mesh_data_geom_subset(
              pathString, nameString, ty, indices);
          }
        }
      }
    }
  }
}
