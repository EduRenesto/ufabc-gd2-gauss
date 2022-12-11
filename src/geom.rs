//! TODO(edu): documentar

use std::collections::{HashMap, HashSet};

use ultraviolet::Vec3;

pub fn compute_neighborhoods(mesh: &tobj::Mesh) -> Vec<HashSet<u32>> {
    // NOTE(edu): a ideia aqui é encontrar a vizinhança de cada
    // vértice da malha.
    //
    // Pra isso, vamos observar que cada face nos dá a informação de
    // vizinhança pra tres vértices de uma vez só. Visitamos cada vértice
    // da face, e adicionamos os vértices adjacentes a uma lista.
    let mut nbhds = HashMap::new();

    // Como estamos trabalhando sobre malhas de triangulos, cada face
    // tem exatamente tres vertices. Portanto o número de faces total
    // será o número de indices / 3.
    let n_faces = mesh.indices.len() / 3;

    for i in 0..n_faces {
        let v1_idx =  mesh.indices[3*i    ];
        let v2_idx =  mesh.indices[3*i + 1];
        let v3_idx =  mesh.indices[3*i + 2];

        {
            let entry = nbhds.entry(v1_idx).or_insert(HashSet::new());

            entry.insert(v2_idx);
            entry.insert(v3_idx);
        }

        {
            let entry = nbhds.entry(v2_idx).or_insert(HashSet::new());

            entry.insert(v1_idx);
            entry.insert(v3_idx);
        }

        {
            let entry = nbhds.entry(v3_idx).or_insert(HashSet::new());

            entry.insert(v2_idx);
            entry.insert(v1_idx);
        }
    }

    let mut ret = vec![HashSet::new(); mesh.positions.len()];

    for (vtx_idx, vtx_nbs_set) in nbhds.into_iter() {
        ret[vtx_idx as usize] = vtx_nbs_set;
    }

    ret
}

pub fn compute_avg_normals(mesh: &tobj::Mesh) -> Vec<Vec3> {
    let mut normals = HashMap::new();

    for i in 0..mesh.indices.len() {
        let vertex_idx =  mesh.indices[i];
        let normal_idx =  mesh.normal_indices[i];

        let entry = normals.entry(vertex_idx).or_insert(Vec::new());

        entry.push(Vec3::new(
            mesh.normals[3 * normal_idx as usize + 0],
            mesh.normals[3 * normal_idx as usize + 1],
            mesh.normals[3 * normal_idx as usize + 2],
        ));
    }

    let mut ret = vec![Vec3::zero(); mesh.positions.len()/3];

    for i in 0..(mesh.positions.len()/3) {
        let vtx_normals = normals
            .entry(i as u32)
            .or_default();

        let mut average = Vec3::zero();

        for normal in vtx_normals.iter() {
            average += *normal;
        }

        average /= vtx_normals.len() as f32;

        ret[i] = average.normalized();
    }

    ret
}
