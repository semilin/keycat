use keycat::{
    Analyzer, Corpus, Layout, MetricAmount, MetricData, NgramType, Nstroke, NstrokeData, Swap,
};

pub fn main() {
    let mut corpus = {
        let mut char_list = "abcdefghijklmnopqrstuvwxyz"
            .chars()
            .map(|c| vec![c, c.to_uppercase().next().unwrap()])
            .collect::<Vec<Vec<char>>>();
        char_list.extend(vec![
            vec![',', '<'],
            vec!['.', '>'],
            vec!['/', '?'],
            vec!['\'', '\"'],
        ]);
        Corpus::with_char_list(&mut char_list)
    };

    // println!("{:?}", corpus.char_list);

    corpus.add_file("./tr.txt").unwrap();

    // for (i, v) in corpus.bigrams.iter().enumerate() {
    //     if *v >= 533 {
    //         let s: String = corpus.uncorpus_bigram(i).iter().collect();
    //         println!("{}. '{}' {}", i, s, v);
    //     }
    // }

    let metrics = vec![NgramType::Bigram, NgramType::Skipgram];
    let strokes = vec![
        NstrokeData::new(Nstroke::Bistroke([0, 1]), vec![MetricAmount::new(0, 1.0)]),
        NstrokeData::new(Nstroke::Bistroke([1, 0]), vec![MetricAmount::new(0, 1.0)]),
        NstrokeData::new(Nstroke::Bistroke([0, 2]), vec![MetricAmount::new(0, 1.0)]),
        NstrokeData::new(Nstroke::Bistroke([2, 0]), vec![MetricAmount::new(0, 1.0)]),
        NstrokeData::new(Nstroke::Bistroke([1, 2]), vec![MetricAmount::new(0, 1.0)]),
        NstrokeData::new(Nstroke::Bistroke([2, 1]), vec![MetricAmount::new(0, 1.0)]),
        NstrokeData::new(Nstroke::Bistroke([3, 4]), vec![MetricAmount::new(0, 1.0)]),
        NstrokeData::new(Nstroke::Bistroke([4, 3]), vec![MetricAmount::new(0, 1.0)]),
        NstrokeData::new(Nstroke::Bistroke([5, 3]), vec![MetricAmount::new(0, 1.0)]),
        NstrokeData::new(Nstroke::Bistroke([3, 5]), vec![MetricAmount::new(0, 1.0)]),
        NstrokeData::new(Nstroke::Bistroke([4, 5]), vec![MetricAmount::new(0, 1.0)]),
        NstrokeData::new(Nstroke::Bistroke([5, 4]), vec![MetricAmount::new(0, 1.0)]),
    ];
    let data = MetricData::from(metrics, strokes, 6);

    let layout = Layout {
        matrix: "fsxlrjhnbvtmzkq'cpwdgue,oa.yi/"
            // matrix: "qazwsxedcrfvtgbyhnujmik,ol.p"
            .chars()
            .map(|c| *corpus.corpus_char(c))
            .collect(),
    };

    let mut analyzer = Analyzer::from(data, corpus, layout);

    println!(
        "{:?}",
        analyzer
            .data
            .strokes
            .iter()
            .map(|x| &x.nstroke)
            .collect::<Vec<_>>()
    );
    println!("{:?}", analyzer.data.position_strokes);

    let swap = Swap::new(1, 4);
    
    println!("{:?}", analyzer.stats);
    analyzer.swap(0, &swap, false);
    println!("{:?}", analyzer.stats);
    analyzer.swap(0, &swap, false);
    println!("{:?}", analyzer.stats);
}
