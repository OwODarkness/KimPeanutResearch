import type { PaperDetail, PaperListItem } from "./types";

export const collectionTitle = "Foundation models";
export const collectionDescription =
  "A working collection on how language models are built, adapted, and grounded.";

export const papers: PaperListItem[] = [
  { title: "Attention Is All You Need", authors: "Ashish Vaswani · Noam Shazeer · Niki Parmar · et al.", venue: "NeurIPS 2017", tags: ["transformers", "architecture"], coverColor: "#e8bb8c", isSelected: true },
  { title: "Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks", authors: "Patrick Lewis · Ethan Perez · Aleksandra Piktus · et al.", venue: "NeurIPS 2020", tags: ["retrieval", "language models"], coverColor: "#a8c5dd" },
  { title: "Scaling Laws for Neural Language Models", authors: "Jared Kaplan · Sam McCandlish · Tom Henighan · et al.", venue: "arXiv 2020", tags: ["scaling", "language models"], coverColor: "#baaed6" },
  { title: "LoRA: Low-Rank Adaptation of Large Language Models", authors: "Edward J. Hu · Yelong Shen · Phillip Wallis · et al.", venue: "ICLR 2022", tags: ["fine-tuning", "efficiency"], coverColor: "#a4cbb7" },
  { title: "Training Compute-Optimal Large Language Models", authors: "Jordan Hoffmann · Sebastian Borgeaud · Arthur Mensch · et al.", venue: "arXiv 2022", tags: ["scaling", "training"], coverColor: "#d4b5a8" },
];

export const selectedPaper: PaperDetail = {
  ...papers[0],
  citationCount: "6,142 citations",
  pageCount: 16,
  abstract: "The dominant sequence transduction models are based on complex recurrent or convolutional neural networks. We propose a new simple network architecture, the Transformer, based solely on attention mechanisms.",
  note: "Key shift: removes recurrence entirely, making training highly parallelizable.",
};
