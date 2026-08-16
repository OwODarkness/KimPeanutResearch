export interface PaperListItem {
  title: string;
  authors: string;
  venue: string;
  tags: string[];
  coverColor: string;
  isSelected?: boolean;
}

export interface PaperDetail extends PaperListItem {
  citationCount: string;
  pageCount: number;
  abstract: string;
  note: string;
}
