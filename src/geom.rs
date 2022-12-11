//! TODO(edu): documentar

use std::collections::{HashMap, HashSet};

use ultraviolet::{Mat3, Vec3, Mat2, Vec2};

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

pub fn compute_tangent_basis(
    mesh: &tobj::Mesh,
    nbhds: &Vec<HashSet<u32>>,
    normals: &Vec<Vec3>,
) -> Vec<Mat3> {
    let mut ret = vec![Mat3::identity(); mesh.positions.len()/3];

    for i in 0..(mesh.positions.len()/3) {
        let nbhds = nbhds.get(i).unwrap();

        if nbhds.len() == 0 {
            continue;
        }

        let n = normals[i];

        // Escolhe um outro vertice arbitrario na vizinhanca do vertice atual
        let a_tilde_idx = *nbhds.iter().next().unwrap() as usize;
        let a_tilde = Vec3::new(
            mesh.positions[3*a_tilde_idx + 0],
            mesh.positions[3*a_tilde_idx + 1],
            mesh.positions[3*a_tilde_idx + 2],
        );

        let a = (a_tilde - n * a_tilde.dot(n)).normalized();

        let b = n.cross(a).normalized();

        // Entao, {a, b, n} é uma base de R^3!!! Em particular, {a, b} é base de TpS!!!!!

        ret[i] = Mat3::new(a, b, n);
    }

    ret
}

pub fn compute_shape_operator(
    mesh: &tobj::Mesh,
    nbhds: &Vec<HashSet<u32>>,
    normals: &Vec<Vec3>,
    tangent_bases: &Vec<Mat3>,
) -> Vec<Mat2> {
    let mut ret = vec![Mat2::identity(); mesh.positions.len()/3];

    for i in 0..(mesh.positions.len()/3) {
        let nbhds = nbhds.get(i).unwrap();

        if nbhds.len() < 3 {
            continue;
        }

        let v = Vec3::new(
            mesh.positions[3*i + 0],
            mesh.positions[3*i + 1],
            mesh.positions[3*i + 2],
        );

        let mut nbhds = nbhds.iter();

        let n = normals[i];

        let nb1_idx = *(nbhds.next().unwrap()) as usize;
        let nb2_idx = *(nbhds.next().unwrap()) as usize;
        let nb3_idx = *(nbhds.next().unwrap()) as usize;

        let nb1_vtx = Vec3::new(
            mesh.positions[3*nb1_idx + 0],
            mesh.positions[3*nb1_idx + 1],
            mesh.positions[3*nb1_idx + 2],
        );
        let nb2_vtx = Vec3::new(
            mesh.positions[3*nb2_idx + 0],
            mesh.positions[3*nb2_idx + 1],
            mesh.positions[3*nb2_idx + 2],
        );
        let nb3_vtx = Vec3::new(
            mesh.positions[3*nb3_idx + 0],
            mesh.positions[3*nb3_idx + 1],
            mesh.positions[3*nb3_idx + 2],
        );

        let tps_basis = tangent_bases[i];
        let tps_basis_t = tps_basis.transposed();

        let nb1_local = tps_basis_t * (nb1_vtx - v);
        let nb2_local = tps_basis_t * (nb2_vtx - v);
        let nb3_local = tps_basis_t * (nb3_vtx - v);

        let nb1_h = n.dot(nb1_vtx - v);
        let nb2_h = n.dot(nb2_vtx - v);
        let nb3_h = n.dot(nb3_vtx - v);

        let U = Mat3::new(
            0.5 * Vec3::new(nb1_local.x.powi(2), nb2_local.x.powi(2), nb3_local.x.powi(2)),
            Vec3::new(nb1_local.x * nb1_local.y, nb2_local.x * nb2_local.y, nb3_local.x * nb3_local.y),
            0.5 * Vec3::new(nb1_local.y.powi(2), nb2_local.y.powi(2), nb3_local.y.powi(2)),
        );

        let F = Vec3::new(nb1_h, nb2_h, nb3_h);

        let X = ((U.transposed() * U).inversed()) * F;

        let S = -1.0 * Mat2::new(
            Vec2::new(X.x, X.y),
            Vec2::new(X.y, X.z),
        );

        ret[i] = S;
    }

    ret
}

pub fn compute_curvatures(
    shape_ops: &Vec<Mat2>,
) -> Vec<(f32, f32)> {
    fn trace(m: &Mat2) -> f32 {
        m.cols[0].x * m.cols[1].y
    }

    shape_ops
        .iter()
        .map(|shape| {
            let k = shape.determinant();
            let h = trace(&shape);

            (k, h)
        })
        .collect()
}
