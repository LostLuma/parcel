import assert from 'assert';
import {
  assertBundles,
  bundle,
  describe,
  distDir,
  it,
  outputFS,
} from '@parcel/test-utils';
import path from 'path';

describe.v2('svg', function () {
  it('should support bundling SVG', async () => {
    let b = await bundle(path.join(__dirname, '/integration/svg/circle.svg'));

    assertBundles(b, [
      {
        name: 'circle.svg',
        assets: ['circle.svg'],
      },
      {
        name: 'other1.html',
        assets: ['other1.html'],
      },
      {
        type: 'svg',
        assets: ['square.svg'],
      },
      {
        name: 'other2.html',
        assets: ['other2.html'],
      },
      {
        type: 'svg',
        assets: ['path.svg'],
      },
      {
        type: 'svg',
        assets: ['gradient.svg'],
      },
      {
        type: 'js',
        assets: ['script.js'],
      },
      {
        type: 'js',
        assets: ['module.js', 'script.js'],
      },
      {
        type: 'css',
        assets: ['style.css'],
      },
    ]);

    let file = await outputFS.readFile(
      b.getBundles().find(b => b.type === 'svg').filePath,
      'utf-8',
    );
    assert(file.includes('<a href="/other1.html">'));
    assert(file.includes('<use href="#circle"'));
    assert(
      file.includes(
        `<use xlink:href="/${path.basename(
          b.getBundles().find(b => b.name.startsWith('square')).filePath,
        )}#square"`,
      ),
    );
    assert(
      file.includes(
        `fill="url('/${path.basename(
          b.getBundles().find(b => b.name.startsWith('gradient')).filePath,
        )}#myGradient')"`,
      ),
    );
    assert(
      file.includes(
        `<script xlink:href="/${path.basename(
          b
            .getBundles()
            .find(b => b.type === 'js' && b.env.sourceType === 'script')
            .filePath,
        )}"`,
      ),
    );
    assert(
      file.includes(
        `<script href="/${path.basename(
          b
            .getBundles()
            .find(b => b.type === 'js' && b.env.sourceType === 'module')
            .filePath,
        )}"`,
      ),
    );
    assert(
      file.includes(
        `<?xml-stylesheet href="/${path.basename(
          b.getBundles().find(b => b.type === 'css').filePath,
        )}"?>`,
      ),
    );
  });

  it('should minify SVG bundles', async function () {
    let b = await bundle(path.join(__dirname, '/integration/svg/circle.svg'), {
      defaultTargetOptions: {
        shouldOptimize: true,
      },
    });

    let file = await outputFS.readFile(
      b.getBundles().find(b => b.type === 'svg').filePath,
      'utf-8',
    );
    assert(!file.includes('comment'));
  });

  it('support SVGO config files', async function () {
    let b = await bundle(
      path.join(__dirname, '/integration/svgo-config/index.html'),
      {
        defaultTargetOptions: {
          shouldOptimize: true,
        },
      },
    );

    let file = await outputFS.readFile(
      b.getBundles().find(b => b.type === 'svg').filePath,
      'utf-8',
    );
    assert(!file.includes('inkscape'));
    assert(file.includes('comment'));
  });

  it('should detect xml-stylesheet processing instructions', async function () {
    let b = await bundle(
      path.join(__dirname, '/integration/svg-xml-stylesheet/img.svg'),
    );

    assertBundles(b, [
      {
        name: 'img.svg',
        assets: ['img.svg'],
      },
      {
        type: 'css',
        assets: ['style1.css'],
      },
      {
        type: 'css',
        assets: ['style3.css'],
      },
    ]);

    let file = await outputFS.readFile(
      b.getBundles().find(b => b.type === 'svg').filePath,
      'utf-8',
    );

    assert(file.includes('<?xml-stylesheet'));
    assert(file.includes('<?xml-not-a-stylesheet'));
  });

  it('should handle inline CSS with @imports', async function () {
    const b = await bundle(
      path.join(__dirname, '/integration/svg-inline-css-import/img.svg'),
    );

    assertBundles(b, [
      {
        type: 'css',
        assets: ['img.svg', 'test.css'],
      },
      {
        type: 'css',
        assets: ['img.svg'],
      },
      {
        name: 'img.svg',
        assets: ['img.svg'],
      },
      {
        type: 'svg',
        assets: ['gradient.svg'],
      },
      {
        type: 'js',
        assets: ['img.svg', 'script.js'],
      },
    ]);

    const svg = await outputFS.readFile(path.join(distDir, 'img.svg'), 'utf8');

    assert(!svg.includes('@import'));
    assert(svg.includes(':root {\n  fill: red;\n}'));
    assert(
      svg.includes(
        `"fill: url(&quot;${path.basename(
          b.getBundles().find(b => b.name.startsWith('gradient')).filePath,
        )}#myGradient&quot;)`,
      ),
    );
    assert(svg.includes('<script>'));
    assert(svg.includes(`console.log('script')`));
    assert(!svg.includes('import '));
  });

  it('should process inline styles using lang', async function () {
    const b = await bundle(
      path.join(__dirname, '/integration/svg-inline-sass/img.svg'),
      {
        defaultTargetOptions: {
          shouldOptimize: true,
        },
      },
    );

    assertBundles(b, [
      {
        type: 'css',
        assets: ['img.svg'],
      },
      {
        name: 'img.svg',
        assets: ['img.svg'],
      },
    ]);

    const svg = await outputFS.readFile(path.join(distDir, 'img.svg'), 'utf8');

    assert(svg.includes('<style>:root{fill:red}</style>'));
  });

  it('should be in separate bundles', async function () {
    const b = await bundle(
      path.join(__dirname, '/integration/svg-multiple/index.js'),
    );

    assertBundles(b, [
      {
        assets: ['index.js', 'bundle-url.js'],
      },
      {
        assets: ['circle.svg'],
      },
      {
        assets: ['square.svg'],
      },
    ]);
  });
});
