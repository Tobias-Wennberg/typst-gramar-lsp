vim.lsp.set_log_level(1) 
vim.lsp.start({
  name = 'typst-spell-lsp',
  cmd = {'./target/debug/typst-lsp'},
})
lsp_onAttach("", 1)
