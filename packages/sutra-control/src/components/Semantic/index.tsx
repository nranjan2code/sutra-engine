import React, { useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  TextField,
  Button,
  Grid,
  Chip,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Alert,
  CircularProgress,
  Tabs,
  Tab,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  IconButton,
  Tooltip,
} from '@mui/material';
import {
  Search as SearchIcon,
  Timeline as TimelineIcon,
  CallSplit as CausalIcon,
  Warning as WarningIcon,
  FilterList as FilterIcon,
  ExpandMore as ExpandMoreIcon,
  ContentCopy as CopyIcon,
  Download as DownloadIcon,
} from '@mui/icons-material';

interface SemanticFilter {
  semantic_types: string[];
  domains: string[];
  min_confidence: number;
  causal_only: boolean;
  required_terms: string[];
}

interface SemanticPath {
  concept_ids: string[];
  confidences: number[];
  total_confidence: number;
  semantic_summary?: string;
}

interface Contradiction {
  concept_id1: string;
  concept_id2: string;
  confidence: number;
}

const SEMANTIC_TYPES = [
  'Rule',
  'Fact',
  'Definition',
  'Hypothesis',
  'Procedure',
  'Question',
  'Negation',
  'Causal',
  'Temporal',
  'Comparison',
  'Unknown',
];

const DOMAINS = [
  'medical',
  'legal',
  'finance',
  'technical',
  'business',
  'education',
  'science',
  'manufacturing',
  'healthcare',
  'security',
  'compliance',
  'engineering',
  'logistics',
  'environment',
  'government',
];

export default function SemanticExplorer() {
  const [activeTab, setActiveTab] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // Path finding state
  const [startQuery, setStartQuery] = useState('');
  const [endQuery, setEndQuery] = useState('');
  const [maxDepth, setMaxDepth] = useState(5);
  const [paths, setPaths] = useState<SemanticPath[]>([]);
  
  // Temporal chain state
  const [temporalStart, setTemporalStart] = useState('');
  const [temporalEnd, setTemporalEnd] = useState('');
  const [afterDate, setAfterDate] = useState('');
  const [beforeDate, setBeforeDate] = useState('');
  const [temporalChains, setTemporalChains] = useState<any[]>([]);
  
  // Causal chain state
  const [causalStart, setCausalStart] = useState('');
  const [causalEnd, setCausalEnd] = useState('');
  const [causalChains, setCausalChains] = useState<any[]>([]);
  
  // Contradiction state
  const [contradictionQuery, setContradictionQuery] = useState('');
  const [contradictions, setContradictions] = useState<Contradiction[]>([]);
  
  // Filter state
  const [filter, setFilter] = useState<SemanticFilter>({
    semantic_types: [],
    domains: [],
    min_confidence: 0.0,
    causal_only: false,
    required_terms: [],
  });

  const handleSemanticPath = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch('/api/semantic/path', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          start_query: startQuery,
          end_query: endQuery,
          max_depth: maxDepth,
          filter: Object.keys(filter).length > 0 ? filter : undefined,
        }),
      });
      
      if (!response.ok) throw new Error('Semantic path query failed');
      
      const data = await response.json();
      setPaths(data.paths || []);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleTemporalChain = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch('/api/semantic/temporal', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          start_query: temporalStart,
          end_query: temporalEnd,
          max_depth: 10,
          after: afterDate || undefined,
          before: beforeDate || undefined,
        }),
      });
      
      if (!response.ok) throw new Error('Temporal chain query failed');
      
      const data = await response.json();
      setTemporalChains(data.chains || []);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleCausalChain = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch('/api/semantic/causal', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          start_query: causalStart,
          end_query: causalEnd,
          max_depth: 5,
        }),
      });
      
      if (!response.ok) throw new Error('Causal chain query failed');
      
      const data = await response.json();
      setCausalChains(data.chains || []);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleContradictions = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch('/api/semantic/contradictions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          query: contradictionQuery,
          max_depth: 3,
        }),
      });
      
      if (!response.ok) throw new Error('Contradiction detection failed');
      
      const data = await response.json();
      setContradictions(data.contradictions || []);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const exportResults = (data: any, filename: string) => {
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
  };

  return (
    <Box sx={{ p: 3 }}>
      <Typography variant="h4" gutterBottom>
        Semantic Reasoning Explorer
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
        Advanced semantic query interface for domain-specific reasoning
      </Typography>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      <Tabs value={activeTab} onChange={(_, v) => setActiveTab(v)} sx={{ mb: 3 }}>
        <Tab icon={<SearchIcon />} label="Semantic Path" />
        <Tab icon={<TimelineIcon />} label="Temporal Chain" />
        <Tab icon={<CausalIcon />} label="Causal Chain" />
        <Tab icon={<WarningIcon />} label="Contradictions" />
      </Tabs>

      {/* Semantic Path Tab */}
      {activeTab === 0 && (
        <Card>
          <CardContent>
            <Grid container spacing={2}>
              <Grid item xs={12} md={5}>
                <TextField
                  fullWidth
                  label="Start Concept"
                  value={startQuery}
                  onChange={(e) => setStartQuery(e.target.value)}
                  placeholder="concept_id or natural language"
                />
              </Grid>
              <Grid item xs={12} md={5}>
                <TextField
                  fullWidth
                  label="End Concept"
                  value={endQuery}
                  onChange={(e) => setEndQuery(e.target.value)}
                  placeholder="concept_id or natural language"
                />
              </Grid>
              <Grid item xs={12} md={2}>
                <TextField
                  fullWidth
                  type="number"
                  label="Max Depth"
                  value={maxDepth}
                  onChange={(e) => setMaxDepth(parseInt(e.target.value))}
                  inputProps={{ min: 1, max: 10 }}
                />
              </Grid>
              
              {/* Semantic Filter */}
              <Grid item xs={12}>
                <Accordion>
                  <AccordionSummary expandIcon={<ExpandMoreIcon />}>
                    <FilterIcon sx={{ mr: 1 }} />
                    <Typography>Semantic Filter (Optional)</Typography>
                  </AccordionSummary>
                  <AccordionDetails>
                    <Grid container spacing={2}>
                      <Grid item xs={12} md={6}>
                        <FormControl fullWidth>
                          <InputLabel>Semantic Types</InputLabel>
                          <Select
                            multiple
                            value={filter.semantic_types}
                            onChange={(e) =>
                              setFilter({ ...filter, semantic_types: e.target.value as string[] })
                            }
                            renderValue={(selected) => (
                              <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                                {selected.map((value) => (
                                  <Chip key={value} label={value} size="small" />
                                ))}
                              </Box>
                            )}
                          >
                            {SEMANTIC_TYPES.map((type) => (
                              <MenuItem key={type} value={type}>
                                {type}
                              </MenuItem>
                            ))}
                          </Select>
                        </FormControl>
                      </Grid>
                      <Grid item xs={12} md={6}>
                        <FormControl fullWidth>
                          <InputLabel>Domains</InputLabel>
                          <Select
                            multiple
                            value={filter.domains}
                            onChange={(e) =>
                              setFilter({ ...filter, domains: e.target.value as string[] })
                            }
                            renderValue={(selected) => (
                              <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                                {selected.map((value) => (
                                  <Chip key={value} label={value} size="small" />
                                ))}
                              </Box>
                            )}
                          >
                            {DOMAINS.map((domain) => (
                              <MenuItem key={domain} value={domain}>
                                {domain}
                              </MenuItem>
                            ))}
                          </Select>
                        </FormControl>
                      </Grid>
                      <Grid item xs={12} md={6}>
                        <TextField
                          fullWidth
                          type="number"
                          label="Min Confidence"
                          value={filter.min_confidence}
                          onChange={(e) =>
                            setFilter({ ...filter, min_confidence: parseFloat(e.target.value) })
                          }
                          inputProps={{ min: 0, max: 1, step: 0.1 }}
                        />
                      </Grid>
                    </Grid>
                  </AccordionDetails>
                </Accordion>
              </Grid>
              
              <Grid item xs={12}>
                <Button
                  variant="contained"
                  onClick={handleSemanticPath}
                  disabled={loading || !startQuery || !endQuery}
                  startIcon={loading ? <CircularProgress size={20} /> : <SearchIcon />}
                  fullWidth
                >
                  Find Semantic Path
                </Button>
              </Grid>
            </Grid>

            {paths.length > 0 && (
              <Box sx={{ mt: 3 }}>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 2 }}>
                  <Typography variant="h6">Results: {paths.length} paths found</Typography>
                  <Button
                    startIcon={<DownloadIcon />}
                    onClick={() => exportResults(paths, 'semantic-paths.json')}
                  >
                    Export JSON
                  </Button>
                </Box>
                {paths.map((path, idx) => (
                  <Card key={idx} sx={{ mb: 2 }}>
                    <CardContent>
                      <Typography variant="subtitle2" gutterBottom>
                        Path {idx + 1} - Confidence: {(path.total_confidence * 100).toFixed(1)}%
                      </Typography>
                      <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 1 }}>
                        {path.concept_ids.map((id, i) => (
                          <React.Fragment key={i}>
                            <Chip label={id.substring(0, 8)} size="small" />
                            {i < path.concept_ids.length - 1 && <Typography>→</Typography>}
                          </React.Fragment>
                        ))}
                      </Box>
                      {path.semantic_summary && (
                        <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                          {path.semantic_summary}
                        </Typography>
                      )}
                    </CardContent>
                  </Card>
                ))}
              </Box>
            )}
          </CardContent>
        </Card>
      )}

      {/* Temporal Chain Tab */}
      {activeTab === 1 && (
        <Card>
          <CardContent>
            <Grid container spacing={2}>
              <Grid item xs={12} md={6}>
                <TextField
                  fullWidth
                  label="Start Concept"
                  value={temporalStart}
                  onChange={(e) => setTemporalStart(e.target.value)}
                />
              </Grid>
              <Grid item xs={12} md={6}>
                <TextField
                  fullWidth
                  label="End Concept"
                  value={temporalEnd}
                  onChange={(e) => setTemporalEnd(e.target.value)}
                />
              </Grid>
              <Grid item xs={12} md={6}>
                <TextField
                  fullWidth
                  type="date"
                  label="After Date"
                  value={afterDate}
                  onChange={(e) => setAfterDate(e.target.value)}
                  InputLabelProps={{ shrink: true }}
                />
              </Grid>
              <Grid item xs={12} md={6}>
                <TextField
                  fullWidth
                  type="date"
                  label="Before Date"
                  value={beforeDate}
                  onChange={(e) => setBeforeDate(e.target.value)}
                  InputLabelProps={{ shrink: true }}
                />
              </Grid>
              <Grid item xs={12}>
                <Button
                  variant="contained"
                  onClick={handleTemporalChain}
                  disabled={loading || !temporalStart || !temporalEnd}
                  startIcon={loading ? <CircularProgress size={20} /> : <TimelineIcon />}
                  fullWidth
                >
                  Find Temporal Chain
                </Button>
              </Grid>
            </Grid>

            {temporalChains.length > 0 && (
              <Box sx={{ mt: 3 }}>
                <Typography variant="h6" gutterBottom>
                  Results: {temporalChains.length} chains found
                </Typography>
                {temporalChains.map((chain, idx) => (
                  <Card key={idx} sx={{ mb: 2 }}>
                    <CardContent>
                      <Typography variant="subtitle2">Chain {idx + 1}</Typography>
                      <pre>{JSON.stringify(chain, null, 2)}</pre>
                    </CardContent>
                  </Card>
                ))}
              </Box>
            )}
          </CardContent>
        </Card>
      )}

      {/* Causal Chain Tab */}
      {activeTab === 2 && (
        <Card>
          <CardContent>
            <Grid container spacing={2}>
              <Grid item xs={12} md={6}>
                <TextField
                  fullWidth
                  label="Cause Concept"
                  value={causalStart}
                  onChange={(e) => setCausalStart(e.target.value)}
                />
              </Grid>
              <Grid item xs={12} md={6}>
                <TextField
                  fullWidth
                  label="Effect Concept"
                  value={causalEnd}
                  onChange={(e) => setCausalEnd(e.target.value)}
                />
              </Grid>
              <Grid item xs={12}>
                <Button
                  variant="contained"
                  onClick={handleCausalChain}
                  disabled={loading || !causalStart || !causalEnd}
                  startIcon={loading ? <CircularProgress size={20} /> : <CausalIcon />}
                  fullWidth
                >
                  Find Causal Chain
                </Button>
              </Grid>
            </Grid>

            {causalChains.length > 0 && (
              <Box sx={{ mt: 3 }}>
                <Typography variant="h6" gutterBottom>
                  Results: {causalChains.length} chains found
                </Typography>
                {causalChains.map((chain, idx) => (
                  <Card key={idx} sx={{ mb: 2 }}>
                    <CardContent>
                      <Typography variant="subtitle2">Chain {idx + 1}</Typography>
                      <pre>{JSON.stringify(chain, null, 2)}</pre>
                    </CardContent>
                  </Card>
                ))}
              </Box>
            )}
          </CardContent>
        </Card>
      )}

      {/* Contradictions Tab */}
      {activeTab === 3 && (
        <Card>
          <CardContent>
            <Grid container spacing={2}>
              <Grid item xs={12}>
                <TextField
                  fullWidth
                  label="Concept to Check"
                  value={contradictionQuery}
                  onChange={(e) => setContradictionQuery(e.target.value)}
                  placeholder="concept_id or natural language"
                />
              </Grid>
              <Grid item xs={12}>
                <Button
                  variant="contained"
                  color="warning"
                  onClick={handleContradictions}
                  disabled={loading || !contradictionQuery}
                  startIcon={loading ? <CircularProgress size={20} /> : <WarningIcon />}
                  fullWidth
                >
                  Detect Contradictions
                </Button>
              </Grid>
            </Grid>

            {contradictions.length > 0 && (
              <Box sx={{ mt: 3 }}>
                <Typography variant="h6" gutterBottom color="warning.main">
                  ⚠️ {contradictions.length} contradictions detected
                </Typography>
                <TableContainer component={Paper}>
                  <Table>
                    <TableHead>
                      <TableRow>
                        <TableCell>Concept 1</TableCell>
                        <TableCell>Concept 2</TableCell>
                        <TableCell>Confidence</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {contradictions.map((c, idx) => (
                        <TableRow key={idx}>
                          <TableCell>{c.concept_id1.substring(0, 16)}</TableCell>
                          <TableCell>{c.concept_id2.substring(0, 16)}</TableCell>
                          <TableCell>{(c.confidence * 100).toFixed(1)}%</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
              </Box>
            )}
            {contradictions.length === 0 && contradictionQuery && !loading && (
              <Alert severity="success" sx={{ mt: 2 }}>
                No contradictions detected
              </Alert>
            )}
          </CardContent>
        </Card>
      )}
    </Box>
  );
}
