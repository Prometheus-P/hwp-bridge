import { useState } from 'react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
function App() {
  const [doc, setDoc] = useState(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [selectedPath, setSelectedPath] = useState('');

  async function handleOpenFile() {
    try {
      if (!isTauri()) {
        setError('파일 열기는 Tauri 앱에서만 지원됩니다. `npm run tauri dev`로 실행하세요.');
        return;
      }

      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'HWP Files',
            extensions: ['hwp']
          }
        ]
      });

      if (!selected) {
        return;
      }

      const path = Array.isArray(selected) ? selected[0] : selected;
      if (!path) {
        return;
      }

      setSelectedPath(path);
      setLoading(true);
      setError('');
      setDoc(null);

      try {
        const result = await invoke('parse_hwp_file', { path });
        setDoc(result);
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    } catch (e) {
      setError(`파일 선택 중 오류가 발생했습니다: ${String(e)}`);
    }
  }

  function paragraphText(paragraph) {
    return (paragraph?.runs || []).map((run) => run.text || '').join('');
  }

  function cellTextFromBlocks(blocks) {
    return (blocks || [])
      .map((block) => {
        switch (block.block_type) {
          case 'paragraph':
            return paragraphText(block);
          case 'raw_text':
            return block.text || '';
          case 'table':
            return `표 ${block.row_count || 0}x${block.col_count || 0}`;
          default:
            return '';
        }
      })
      .filter((text) => text && text.trim().length > 0)
      .join('\n');
  }

  function tableHasCellContent(rows) {
    return (rows || []).some((row) =>
      (row || []).some((cell) => (cell.blocks || []).length > 0)
    );
  }

  function deriveTableRows(table, followingBlocks) {
    const originalRows = table.rows || [];
    if (tableHasCellContent(originalRows)) {
      return { rows: originalRows, consumed: 0 };
    }

    const rowCount = table.row_count || 0;
    const colCount = table.col_count || 0;
    const totalCells = rowCount * colCount;
    if (!totalCells) {
      return { rows: originalRows, consumed: 0 };
    }

    const texts = [];
    let consumed = 0;
    for (const next of followingBlocks) {
      if (next?.type !== 'Paragraph') {
        break;
      }
      texts.push(paragraphText(next));
      consumed += 1;
      if (texts.length >= totalCells) {
        break;
      }
    }

    if (texts.length === 0) {
      return { rows: originalRows, consumed: 0 };
    }

    const rows =
      originalRows.length === rowCount
        ? originalRows.map((row) =>
            row.map((cell) => ({
              ...cell,
              blocks: [],
            }))
          )
        : Array.from({ length: rowCount }, () =>
            Array.from({ length: colCount }, () => ({ blocks: [] }))
          );

    let cursor = 0;
    for (let r = 0; r < rowCount; r += 1) {
      for (let c = 0; c < colCount; c += 1) {
        const text = texts[cursor++] || '';
        if (text.trim().length > 0) {
          rows[r][c].blocks = [{ block_type: 'raw_text', text }];
        }
      }
    }

    return { rows, consumed };
  }

  function renderTable(table, index, rows) {
    return (
      <div key={`t-${index}`} className="table-card">
        <div className="table-title">
          표 {table.row_count}x{table.col_count}
        </div>
        <div className="table-grid">
          {(rows || []).map((row, rIdx) => (
            <div key={`tr-${index}-${rIdx}`} className="table-row">
              {row.map((cell, cIdx) => (
                <div key={`tc-${index}-${rIdx}-${cIdx}`} className="table-cell">
                  {cellTextFromBlocks(cell.blocks) || '\u00a0'}
                </div>
              ))}
            </div>
          ))}
        </div>
      </div>
    );
  }

  function renderBlock(block, index) {
    if (!block || !block.type) {
      return null;
    }

    switch (block.type) {
      case 'Paragraph': {
        const text = paragraphText(block);
        return (
          <p key={`p-${index}`} className="paragraph">
            {text || '\u00a0'}
          </p>
        );
      }
      case 'Table': {
        return renderTable(block, index, block.rows || []);
      }
      case 'Chart': {
        const chartTitle = block.title || '차트';
        return (
          <div key={`c-${index}`} className="chart-card">
            <div className="chart-title">{chartTitle}</div>
            <div className="chart-meta">
              {block.chart_type && <span>유형: {block.chart_type}</span>}
              <span>BinData ID: {block.bin_data_id ?? '-'}</span>
              <span>Stream: {block.stream_type ?? 'unknown'}</span>
            </div>
            {block.note && <p className="chart-note">{block.note}</p>}
            {block.data_grid &&
              renderTable(
                block.data_grid,
                `${index}-chart`,
                block.data_grid.rows || [],
              )}
          </div>
        );
      }
      default:
        return (
          <div key={`b-${index}`} className="block-placeholder">
            {block.type} 블록
          </div>
        );
    }
  }

  return (
    <div className="app">
      <header className="hero">
        <div className="hero-text">
          <p className="eyebrow">HwpBridge GUI</p>
          <h1>HWP 문서를 빠르게 읽고 구조화합니다.</h1>
          <p className="subhead">
            BodyText를 파싱해 문단과 표를 구조화된 뷰로 표시합니다.
          </p>
        </div>
        <div className="hero-actions">
          <button className="primary" onClick={handleOpenFile} disabled={loading}>
            {loading ? '파싱 중...' : 'HWP 파일 열기'}
          </button>
          <p className="path">{selectedPath || '선택된 파일 없음'}</p>
        </div>
      </header>

      <main className="content-area">
        {error && (
          <div className="error-card">
            <strong>오류</strong>
            <span>{error}</span>
          </div>
        )}

        {!doc && !loading && !error && (
          <div className="empty-state">
            <h2>문서를 열어주세요</h2>
            <p>.hwp 파일을 선택하면 내용이 여기에 표시됩니다.</p>
          </div>
        )}

        {doc && (
          <section className="document">
            <div className="metadata-card">
              <h2>{doc.metadata?.title || '제목 없음'}</h2>
              <div className="meta-grid">
                <div>
                  <span className="meta-label">저자</span>
                  <span>{doc.metadata?.author || '알 수 없음'}</span>
                </div>
                <div>
                  <span className="meta-label">생성일</span>
                  <span>{doc.metadata?.created_at || '-'}</span>
                </div>
                <div>
                  <span className="meta-label">섹션 수</span>
                  <span>{doc.sections?.length || 0}</span>
                </div>
              </div>
            </div>

            <div className="sections">
              {(doc.sections || []).map((section, sIdx) => (
                <article key={`s-${sIdx}`} className="section-card">
                  <div className="section-header">
                    <h3>Section {sIdx + 1}</h3>
                    <span className="section-count">
                      블록 {section.content?.length || 0}
                    </span>
                  </div>
                  <div className="section-body">
                    {(() => {
                      const content = section.content || [];
                      const rendered = [];
                      let i = 0;
                      while (i < content.length) {
                        const block = content[i];
                        if (block?.type === 'Table') {
                          const { rows, consumed } = deriveTableRows(
                            block,
                            content.slice(i + 1)
                          );
                          rendered.push(renderTable(block, i, rows));
                          i += 1 + consumed;
                          continue;
                        }
                        rendered.push(renderBlock(block, i));
                        i += 1;
                      }
                      return rendered;
                    })()}
                  </div>
                </article>
              ))}
            </div>
          </section>
        )}
      </main>
    </div>
  );
}

export default App;
