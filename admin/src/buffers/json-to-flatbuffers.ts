import * as flatbuffers from 'flatbuffers';
import {
  Align,
  Code,
  Codespan,
  Del,
  Document,
  Em,
  Heading,
  Hr,
  Br,
  Html,
  Image,
  Link,
  List,
  ListItem,
  Paragraph,
  Space,
  Strong,
  Table,
  TableCell,
  TableRow,
  Text,
  Token,
  TokenType,
} from './pagezest-markdown';

export function buildFlatBufferFromJson(json: any[]): Uint8Array {
  const builder = new flatbuffers.Builder(1024);

  const tokenOffsets = json.map((node) => buildToken(builder, node));

  const tokensVector = Document.createTokensVector(builder, tokenOffsets);
  Document.startDocument(builder);
  Document.addTokens(builder, tokensVector);
  const docOffset = Document.endDocument(builder);
  builder.finish(docOffset);

  return builder.asUint8Array();
}

function buildToken(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  let type: TokenType;
  let valueOffset: flatbuffers.Offset;

  switch (node.type) {
    case 'heading':
      type = TokenType.HEADING;
      valueOffset = buildHeading(builder, node);
      break;
    case 'paragraph':
      type = TokenType.PARAGRAPH;
      valueOffset = buildParagraph(builder, node);
      break;
    case 'list':
      type = TokenType.LIST;
      valueOffset = buildList(builder, node);
      break;
    case 'list_item':
      type = TokenType.LIST_ITEM;
      valueOffset = buildListItem(builder, node);
      break;
    case 'space':
      type = TokenType.SPACE;
      valueOffset = Space.createSpace(builder);
      break;
    case 'hr':
      type = TokenType.HR;
      valueOffset = Hr.createHr(builder);
      break;
    case 'br':
      type = TokenType.BR;
      valueOffset = Br.createBr(builder);
      break;
    case 'table':
      type = TokenType.TABLE;
      valueOffset = buildTable(builder, node);
      break;
    case 'text':
      type = TokenType.TEXT;
      valueOffset = buildText(builder, node);
      break;
    case 'strong':
      type = TokenType.STRONG;
      valueOffset = buildStrong(builder, node);
      break;
    case 'em':
      type = TokenType.EM;
      valueOffset = buildEm(builder, node);
      break;
    case 'del':
      type = TokenType.DEL;
      valueOffset = buildDel(builder, node);
      break;
    case 'link':
      type = TokenType.LINK;
      valueOffset = buildLink(builder, node);
      break;
    case 'image':
      type = TokenType.IMAGE;
      valueOffset = buildImage(builder, node);
      break;
    case 'html':
      type = TokenType.HTML;
      valueOffset = buildHtml(builder, node);
      break;
    case 'code':
      type = TokenType.CODE;
      valueOffset = buildCode(builder, node);
      break;
    case 'codespan':
      type = TokenType.CODESPAN;
      valueOffset = buildCodespan(builder, node);
      break;
    case 'blockquote':
        type = TokenType.BLOCKQUOTE;
      valueOffset = buildParagraph(builder, node);
      break;
    default:
      throw new Error(`Unsupported node type: ${node.type}`);
  }

  Token.startToken(builder);
  Token.addType(builder, type);
  Token.addValueType(builder, type);
  Token.addValue(builder, valueOffset);
  return Token.endToken(builder);
}

function buildText(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const textOffset = builder.createString(node.text);
  const escaped = !!node.escaped;

  let tokensOffset = 0;
  if (node.tokens && Array.isArray(node.tokens)) {
    const tokenOffsets = node.tokens.map(t => buildToken(builder, t));
    Text.startTokensVector(builder, tokenOffsets.length);
    for (let i = tokenOffsets.length - 1; i >= 0; i--) {
      builder.addOffset(tokenOffsets[i]);
    }
    tokensOffset = builder.endVector();
  }

  Text.startText(builder);
  Text.addText(builder, textOffset);
  if (tokensOffset) Text.addTokens(builder, tokensOffset);
  Text.addEscaped(builder, escaped);
  return Text.endText(builder);
}

function buildTokensVector(builder: flatbuffers.Builder, nodes: any[]): flatbuffers.Offset {
  const offsets = nodes.map((n) => buildToken(builder, n));
  return Paragraph.createTokensVector(builder, offsets);
}

function buildHeading(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const textOffset = builder.createString(node.text || '');
  const tokensOffset = buildTokensVector(builder, node.tokens || []);
  Heading.startHeading(builder);
  Heading.addText(builder, textOffset);
  Heading.addDepth(builder, node.depth || 0);
  Heading.addTokens(builder, tokensOffset);
  return Heading.endHeading(builder);
}

function buildParagraph(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const tokensOffset = buildTokensVector(builder, node.tokens || []);
  const text = builder.createString(node.text || '');
  Paragraph.startParagraph(builder);
  Paragraph.addText(builder, text);
  Paragraph.addTokens(builder, tokensOffset);
  return Paragraph.endParagraph(builder);
}

function buildList(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const items = (node.items || []).map((n) => buildToken(builder, n));
  const itemsOffset = List.createItemsVector(builder, items);
  List.startList(builder);
  List.addOrdered(builder, node.ordered || false);
  List.addStart(builder, node.start || 0);
  //List.addTight(builder, node.tight || false);
  List.addItems(builder, itemsOffset);
  return List.endList(builder);
}

function buildListItem(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const tokensOffset = buildTokensVector(builder, node.tokens || []);
  ListItem.startListItem(builder);
  ListItem.addTask(builder, !!node.task);
  ListItem.addChecked(builder, !!node.checked);
  ListItem.addTokens(builder, tokensOffset);
  return ListItem.endListItem(builder);
}

function buildTable(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  console.log(node.align);
  const alignments = (node.align || []).map((a: string) => Align[a?.toUpperCase()] || Align.NONE);
  const alignOffset = Table.createAlignVector(builder, alignments);
  const headerCells = (node.header || []).map((c: any) => buildTableCell(builder, c));
  const headerOffset = Table.createHeaderVector(builder, headerCells);
  const rows = (node.rows || []).map((row: any[]) => {
    const cells = row.map((cell) => buildTableCell(builder, cell));
    const vec = TableRow.createCellsVector(builder, cells);
    TableRow.startTableRow(builder);
    TableRow.addCells(builder, vec);
    return TableRow.endTableRow(builder);
  });
  const rowsOffset = Table.createRowsVector(builder, rows);
  Table.startTable(builder);
  Table.addAlign(builder, alignOffset);
  Table.addHeader(builder, headerOffset);
  Table.addRows(builder, rowsOffset);
  return Table.endTable(builder);
}

function buildTableCell(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const tokensOffset = buildTokensVector(builder, node.tokens || []);
  const text = builder.createString(node.text || '');
  const header = builder.createString(node.header || 0);
  TableCell.startTableCell(builder);
  TableCell.addTokens(builder, tokensOffset);
  TableCell.addText(builder, text);
  TableCell.addHeader(builder, header);
  return TableCell.endTableCell(builder);
}

function buildStrong(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const tokensOffset = buildTokensVector(builder, node.tokens || []);
  Strong.startStrong(builder);
  Strong.addTokens(builder, tokensOffset);
  return Strong.endStrong(builder);
}

function buildEm(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const tokensOffset = buildTokensVector(builder, node.tokens || []);
  Em.startEm(builder);
  Em.addTokens(builder, tokensOffset);
  return Em.endEm(builder);
}

function buildDel(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const tokensOffset = buildTokensVector(builder, node.tokens || []);
  Del.startDel(builder);
  Del.addTokens(builder, tokensOffset);
  return Del.endDel(builder);
}

function buildLink(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const hrefOffset = builder.createString(node.href || '');
  const titleOffset = builder.createString(node.title || '');
  const tokensOffset = buildTokensVector(builder, node.tokens || []);
  Link.startLink(builder);
  Link.addHref(builder, hrefOffset);
  Link.addTitle(builder, titleOffset);
  Link.addTokens(builder, tokensOffset);
  return Link.endLink(builder);
}

function buildImage(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const hrefOffset = builder.createString(node.href || '');
  const titleOffset = builder.createString(node.title || '');
  const textOffset = builder.createString(node.text || '');
  const tokensOffset = buildTokensVector(builder, node.tokens || []);
  Image.startImage(builder);
  Image.addHref(builder, hrefOffset);
  Image.addTitle(builder, titleOffset);
  Image.addText(builder, textOffset);
  Image.addTokens(builder, tokensOffset);
  return Image.endImage(builder);
}

function buildHtml(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const val = builder.createString(node.text || '');
  Html.startHtml(builder);
  Html.addText(builder, val);
  return Html.endHtml(builder);
}

function buildCode(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const codeOffset = builder.createString(node.text || '');
  const langOffset = builder.createString(node.lang || '');
  const preOffset = builder.createString(node.pre || false);
  const tokensOffset = buildTokensVector(builder, node.tokens || []);
  Code.startCode(builder);
  Code.addText(builder, codeOffset);
  Code.addLang(builder, langOffset);
  Code.addPre(builder, preOffset);
  Code.addTokens(builder, tokensOffset);
  return Code.endCode(builder);
}

function buildCodespan(builder: flatbuffers.Builder, node: any): flatbuffers.Offset {
  const textOffset = builder.createString(node.text || '');
  const tokensOffset = buildTokensVector(builder, node.tokens || []);
  Codespan.startCodespan(builder);
  Codespan.addText(builder, textOffset);
  Codespan.addTokens(builder, tokensOffset);
  return Codespan.endCodespan(builder);
}

