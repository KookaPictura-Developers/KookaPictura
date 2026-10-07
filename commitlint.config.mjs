export default {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'type-enum': [
      2,
      'always',
      [
        'feat',
        'fix',
        'docs',
        'style',
        'refactor',
        'perf',
        'test',
        'build',
        'ci',
        'chore',
        'revert',
      ],
    ],
    // Subjects are prose and may start with an acronym or proper noun
    // ("RLE row padding"); config-conventional's sentence-case rule rejects
    // every uppercase-first subject. The repo has no case requirement.
    'subject-case': [0],
    // The repository contract governs the header only; body/footer wrapping and
    // long "Source: <url>" footers are legitimate.
    'body-max-line-length': [0],
    'footer-max-line-length': [0],
    'header-max-length': [2, 'always', 150],
  },
};
