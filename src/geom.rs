use std::collections::{HashMap, HashSet};

pub fn compute_curvatures(mesh: &tobj::Mesh) {
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

    println!("{:?}", nbhds);
}
