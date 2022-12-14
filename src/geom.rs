//! Esse arquivo contém a parte interessante do projeto. Aqui
//! é feito o cálculo da (aproximação da) geometria intrínseca da malha
//! triangular.
//!
//! Nossa idéia é considerar a malha triangular como uma amostragem de uma
//! superfície regular arbitrária, tentar recuperar uma parametrização
//! local (a saber, local para cada vértice) da malha original, e a partir
//! disso calcular o Shape Operator, que nada mais é que a diferencial
//! da aplicação de Gauss para aquela parametrização. A partir disso, teremos
//! uma representação numérica da matriz do shape operator, e a partir disso
//! podemos calcular as curvaturas que queremos.
//!
//! Note que só conseguiremos uma aproximação para a superfície original, e
//! portanto *haverão artefatos* nos cálculos. A aproximação irá depender da
//! geometria da malha triangular original, sendo que quanto maior a resolução,
//! maior a convergência da solução aos valores reais corretos. Provavelmente
//! é possível melhorar a aproximação que fizemos aqui, mas requeriria mais
//! computações (como considerar dois níveis de vizinhança para cada vértice),
//! ou uma álgebra mais refinada (como aumentar o número de pontos usados no
//! ajuste).
//!
//! Para os fins do projeto atual, considero o resultado corrente satisfatório.
//!
//! ## Descrição da malha triangular
//!
//! Antes de discutirmos o método, é interessante descrever como a malha
//! triangular é representada.
//!
//! Utilizamos a biblioteca [`tobj`] para carregar arquivos `.obj` contendo
//! os modelos 3D. Esse formato de arquivo é padrão na prática de computação
//! gráfica, e pode ser construído com inúmeras ferramentas, como o Blender.
//!
//! O arquivo contém os seguintes dados:
//!
//! - Uma lista de todos os vértices do modelo (pontos em R^3);
//! - Uma lista de todas os vetores normais do modelo (vetores em R^3);
//! - Uma lista de todas as *faces* do modelo. Isto é, como os vértices
//!   e as normais se conectam.
//!
//! Dada essa representação, trabalhamos muito com *índices*. Muitas vezes,
//! para nos referirmos a um ponto $p$, utilizaremos o índice de $p$ na lista
//! de vértices, ao invés de um objeto que realmente represente o seu valor. É,
//! inclusive, desta maneira que as faces são descritas.
//!
//! Cada face é composta de três duplas, onde cada dupla contém um vértice
//! e uma normal. Cada dupla representa um dos pontos que compoem o triângulo.
//!
//! Utilizaremos muito a informação das faces para calcular, especialmente, as
//! vizinhanças e as normais médias, como veremos a frente.
//!
//! ## Obtendo a parametrização local
//!
//! Para conseguir a aproximação local, tentamos *ajustar um parabolóide
//! a cada vizinhança*. Ou seja, para cada vértice, tentamos calcular os
//! parâmetros de um parabolóide tal que ele a) seja centrado no vértice
//! considerado e b) contenha em sua imagem um número fixo de vizinhos
//! desse vértice. Para facilitar os cálculos, consideramos 3 vizinhos.
//!
//! Para encontrar tal parametrização, se $p \in \mathbb{R}^3$ é o vértice
//! considerado, olhamos para o plano tangente em $p$, o $T_pS$. Encontraremos
//! uma base para esse plano (diga, $\{ v_1, v_2 \}$), e notaremos que $\{ v_1,
//! v_2, N\}$ onde $N$ é a normal associada ao vértice $p$ é uma base para o
//! $\mathbb{R}^3$ inteiro. A partir disso, utilizaremos o método de quadrados
//! mínimos para encontrar os parâmetros do parabolóide dado em termos dessas
//! bases.
//!
//! Sabendo a descrição total do parabolóide aproximado, fica fácil calcularmos
//! os parâmetros da geometria intrínseca, o que nos permite calcular os valores
//! que queremos.
//!
//! ## Fluxo do cálculo
//!
//! Em cada função, é explicada a ideia por trás e como o cálculo é feito.
//! A ordem lógica de leitura é a seguinte:
//!
//! 1. [`compute_neighborhoods`]: dada a descrição da malha triangular,
//!    encontra as vizinhanças de cada vértice.
//! 2. [`compute_avg_normals`]: calcula a "normal média" para cada vértice.
//! 3. [`compute_tangent_basis`]: para cada vértice p, calcula uma base de TpS.
//! 4. [`compute_shape_operator`]: calcula a matriz que representa o Shape Operator
//!    para cada vértice.
//! 5. [`compute_curvatures`]: a partir das matrizes dos Shape Operators, calcula
//!    as curvaturas gaussianas e médias.
//!
//! TODO(edu): trocar verbatim LaTeX com unicode para renderizar no RustDoc

use std::collections::{HashMap, BTreeSet};

use ultraviolet::{Mat3, Vec3, Mat2, Vec2};

/// Calcula as vizinhanças imediatas de cada vértice.
///
/// Para isso, vamos observar que cada face nos dá a informação de
/// vizinhança pra tres vértices de uma vez só. Visitamos cada vértice
/// da face, e adicionamos os vértices adjacentes a uma lista.
pub fn compute_neighborhoods(mesh: &tobj::Mesh) -> Vec<BTreeSet<u32>> {
    // Estrutura que armazena associações (índice do vértice, conjunto de adjacências)
    let mut nbhds = HashMap::new();

    // Como estamos trabalhando sobre malhas de triangulos, cada face
    // tem exatamente tres vertices. Portanto o número de faces total
    // será o número de indices / 3.
    let n_faces = mesh.indices.len() / 3;

    for i in 0..n_faces {
        // Aqui, estamos olhando para a face de índice i. Ela é composta
        // dos vértices v1, v2 e v3.

        let v1_idx =  mesh.indices[3*i    ];
        let v2_idx =  mesh.indices[3*i + 1];
        let v3_idx =  mesh.indices[3*i + 2];

        // Olha para o vértice v1, e adiciona v2 e v3 a sua lista de
        // adjacências.
        {
            let entry = nbhds.entry(v1_idx).or_insert(BTreeSet::new());

            entry.insert(v2_idx);
            entry.insert(v3_idx);
        }

        // Olha para o vértice v2, e adiciona v1 e v3 a sua lista de
        // adjacências.
        {
            let entry = nbhds.entry(v2_idx).or_insert(BTreeSet::new());

            entry.insert(v1_idx);
            entry.insert(v3_idx);
        }

        // Olha para o vértice v3, e adiciona v1 e v2 a sua lista de
        // adjacências.
        {
            let entry = nbhds.entry(v3_idx).or_insert(BTreeSet::new());

            entry.insert(v1_idx);
            entry.insert(v2_idx);
        }
    }

    // Housekeeping só para retornar os dados numa estrutura de dados
    // mais amigável.
    let mut ret = vec![BTreeSet::new(); mesh.positions.len()];

    for (vtx_idx, vtx_nbs_set) in nbhds.into_iter() {
        ret[vtx_idx as usize] = vtx_nbs_set;
    }

    ret
}

/// Calcula as normais médias para cada vértice.
///
/// Note que cada vértice pode ter mais de um vetor normal associado,
/// dada a descrição das faces. Isso porque um vértice pode ser compartilhado
/// por várias faces (e, inclusive, dependemos disso mais a frente), e pode
/// representar uma normal diferente em cada face.
///
/// Por outro lado, para calcularmos a base do plano tangente de cada vértice,
/// precisaremos de um único valor normal para cada vértice. Para tal, só olhamos
/// para todas as faces que contém cada ponto, e tomamos a média dos vetores
/// normais associados.
///
/// Aqui é uma oportunidade de melhoria da precisão do programa. Podemos tomar a
/// média ponderada considerando a área de cada triângulo, por exemplo. Mas,
/// aqui apenas tomamos a média aritmética.
pub fn compute_avg_normals(mesh: &tobj::Mesh) -> Vec<Vec3> {
    let mut normals = HashMap::new();

    for i in 0..mesh.indices.len() {
        // Olhamos para cada vértice considerando cada triângulo, e acumulamos
        // os vetores normais associados a cada vértice.
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

    // Cálculo da média e housekeeping.
    for i in 0..(mesh.positions.len()/3) {
        let vtx_normals = normals
            .entry(i as u32)
            .or_default();

        let mut average = Vec3::zero();

        for normal in vtx_normals.iter() {
            average += *normal;
        }

        // Tiramos a média...
        average /= vtx_normals.len() as f32;

        // ...e normalizamos.
        ret[i] = average.normalized();
    }

    ret
}

/// Calcula uma base para o plano tangente associado a cada vértice.
///
/// Note que o vetor normal (calculado no método [`compute_avg_normals`]) médio,
/// junto do vértice em si, determina completamente o plano tangente associado
/// ao ponto. Como queremos fazer contas, resta descobrir uma base para tal plano.
///
/// A ideia é simples: tome um vetor a' qualquer, tal que a' não seja paralelo
/// ao plano tangente. Agora, considere a projeção de a' no plano tangente (o que
/// podemos fazer, a partir do vetor normal), e chame de a tal projeção normalizada.
/// Note que tal vetor a faz parte do plano tangente. Então, tome b como sendo a
/// normalização do produto vetorial de a e n. Com essa construção, { a, b } será
/// uma base para o plano tangente.
///
/// Para facilitarmos as computações, retornamos, na verdade, bases para todo o R^3.
/// Tais bases estão na forma de matrizes 3x3, onde cada coluna na matriz é um dos
/// vetores da base, representados por sua vez na base canônica do R^3.
pub fn compute_tangent_basis(
    mesh: &tobj::Mesh,
    nbhds: &Vec<BTreeSet<u32>>,
    normals: &Vec<Vec3>,
) -> Vec<Mat3> {
    let mut ret = vec![Mat3::identity(); mesh.positions.len()/3];

    for i in 0..(mesh.positions.len()/3) {
        // Seja i o vértice p da malha.
        //
        // Seja `nbhds` o conjunto de vértices adjacentes a p.
        let nbhds = nbhds.get(i).unwrap();

        // Seja `n` o vetor normal associado ao vértice p.
        let n = normals[i];

        // Escolhe um outro vertice arbitrario na vizinhanca do vertice atual
        let a_tilde_idx = *nbhds.iter().next().unwrap() as usize;
        let a_tilde = Vec3::new(
            mesh.positions[3*a_tilde_idx + 0],
            mesh.positions[3*a_tilde_idx + 1],
            mesh.positions[3*a_tilde_idx + 2],
        );

        // Calcula a projeção de a' no plano tangente
        let a = (a_tilde - n * a_tilde.dot(n)).normalized();

        // Calcula o vetor b
        let b = n.cross(a).normalized();

        // Entao, {a, b, n} é uma base de R^3!!! Em particular, {a, b} é base de TpS!!!!!
        ret[i] = Mat3::new(a, b, n);
    }

    ret
}

/// Calcula o Shape Operator para cada vértice.
///
/// Nesse método é feito o cálculo da aproximação propriamente dito.
///
/// Consideramos os vértices, as normais e as bases dos planos tangentes
/// calculados acima e fazemos o ajuste de um parabolóide tal que seja
/// definido a partir do plano tangente e contenha os vértices vizinhos.
///
/// Vamos considerar o parabolóide
///
/// $$x(u,v) = \frac{1}{2} \left( au^2 + 2buv + cv^2 \right)$$
///
/// Se $p$ é o vértice que estamos considerando, sejam $p_i$'s os vértices
/// adjacentes a $p$. Escreveremos cada $p_i$ tal que $p_i = x(u_i, v_i)$.
///
/// Fazendo contas, chegaremos a um sistema de equações. Escrevendo esse
/// sistema na forma matricial:
///
/// - $U$ é a matriz 3x3 tal que as linhas de U são vetores da forma
///   $(u_i^2/2, u_i v_i, v_i^2/2)$
/// - $X$ é o vetor coluna contendo os coeficientes $(a, b, c)$ do parabolóide
/// - $F$ é o vetor coluna dos coeficientes em $N$ de cada $p_i$.
///
/// O sistema é representado pela equação $UX = F$, e queremos descobrir $X$.
/// De fato, $X = U^-1 F$. Mas, na prática, usamos a pseudo-inversa para
/// considerar os casos em que $det U = 0$ (o que pode acontecer dada as
/// construções que fizemos). Chegaremos em $X = ((U' U)^-1)U' F$.
///
/// Tendo $X$, a parametrização do parabolóide fica completamente determinada,
/// e podemos fazer contas e chegar na matriz do shape operator, o que é o desejado.
pub fn compute_shape_operator(
    mesh: &tobj::Mesh,
    nbhds: &Vec<BTreeSet<u32>>,
    tangent_bases: &Vec<Mat3>,
) -> Vec<Mat2> {
    let mut ret = vec![Mat2::identity(); mesh.positions.len()/3];

    // Seja v o vértice de índice i.
    for i in 0..(mesh.positions.len()/3) {
        let nbhds = nbhds.get(i).unwrap();

        let v = Vec3::new(
            mesh.positions[3*i + 0],
            mesh.positions[3*i + 1],
            mesh.positions[3*i + 2],
        );

        let mut nbhds = nbhds.iter();

        // Considere os vizinhos v_1, v_2, v_3 de v.
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

        // Observa a base de TvS
        let tps_basis = tangent_bases[i];
        let tps_basis_t = tps_basis.transposed();

        // Calcula as coordenadas dos vizinhos na base do TvS.
        let nb1_local = tps_basis_t * (nb1_vtx - v);
        let nb2_local = tps_basis_t * (nb2_vtx - v);
        let nb3_local = tps_basis_t * (nb3_vtx - v);

        // Extrai o vetor tangente da matriz base do TvS.
        let n = tps_basis.cols[2];

        // Calcula a distância de cada vizinho até o plano tangente.
        // Esses serão valores alcançados pela imagem do parabolóide.
        let nb1_h = n.dot(nb1_vtx - v);
        let nb2_h = n.dot(nb2_vtx - v);
        let nb3_h = n.dot(nb3_vtx - v);

        // Monta a matriz U dos parâmetros do parabolóide.
        let U = Mat3::new(
            0.5 * Vec3::new(nb1_local.x.powi(2), nb2_local.x.powi(2), nb3_local.x.powi(2)),
            Vec3::new(nb1_local.x * nb1_local.y, nb2_local.x * nb2_local.y, nb3_local.x * nb3_local.y),
            0.5 * Vec3::new(nb1_local.y.powi(2), nb2_local.y.powi(2), nb3_local.y.powi(2)),
        );

        // Monta a matriz F dos valores alcançados.
        let F = Vec3::new(nb1_h, nb2_h, nb3_h);

        // Por fim, calcula a matriz dos coeficientes que determinam
        // completamente a parametrização.
        let X = (((U.transposed() * U).inversed()) * U.transposed()) * F;

        // Monta a matriz do shape operator. Ela é da forma
        //
        // _ | a b |
        //   | b c |
        let S = -1.0 * Mat2::new(
            Vec2::new(X.x, X.y),
            Vec2::new(X.y, X.z),
        );

        ret[i] = S;
    }

    ret
}

/// Calcula as curvaturas para cada vértice.
///
/// O cálculo é feito levando em consideração que, se $S$ é
/// a matriz do Shape Operator,
///
/// - $K = det(S)$
/// - $H = tr(S)$
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
