FROM ubuntu:24.04 AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libfreetype6 \
    libjpeg62 \
    libpng16-16t64 \
    libtiff6 \
    libopenjp2-7 \
    poppler-utils \
    && rm -rf /var/lib/apt/lists/*

FROM rust:latest AS rust-build

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
RUN PDFIUM_ARCH=$([ "$TARGETARCH" = "arm64" ] && echo "arm64" || echo "x64") && \
    curl -L -o /tmp/pdfium.tgz \
    "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7520/pdfium-linux-${PDFIUM_ARCH}.tgz" && \
    mkdir -p /opt/pdfium && \
    tar -xzf /tmp/pdfium.tgz -C /opt/pdfium && \
    rm /tmp/pdfium.tgz

FROM rust-build AS pdfium

WORKDIR /build/pdfium
COPY src/pdfium/Cargo.toml src/pdfium/Cargo.lock* ./
COPY src/pdfium/src ./src

RUN cp /opt/pdfium/lib/libpdfium.so /usr/lib/ && ldconfig
RUN cargo build --release

FROM base AS mupdf-build

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    libfreetype6-dev \
    libjpeg-dev \
    libharfbuzz-dev \
    zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
RUN curl -L -o mupdf.tar.gz \
    "https://mupdf.com/downloads/archive/mupdf-1.26.3-source.tar.gz" \
    && tar -xzf mupdf.tar.gz \
    && rm mupdf.tar.gz

WORKDIR /build/mupdf-1.26.3-source
RUN make HAVE_X11=no HAVE_GLUT=no -j$(nproc)

FROM base AS ghostscript-build

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    libfreetype6-dev \
    libjpeg-dev \
    libpng-dev \
    libtiff-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
RUN curl -L -o gs.tar.gz \
    "https://github.com/ArtifexSoftware/ghostpdl-downloads/releases/download/gs10060/ghostscript-10.06.0.tar.gz" \
    && tar -xzf gs.tar.gz \
    && rm gs.tar.gz

WORKDIR /build/ghostscript-10.06.0
RUN ./configure && make -j$(nproc)

FROM node:20-bookworm-slim AS pdfjs

WORKDIR /opt/pdfjs
COPY src/pdfjs/package.json src/pdfjs/package-lock.json* ./
RUN npm ci
COPY src/pdfjs/pdfjs_render.mjs ./

FROM base AS pdfbox-download

RUN apt-get update && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*
RUN curl -L -o /tmp/pdfbox.jar \
    "https://archive.apache.org/dist/pdfbox/3.0.6/pdfbox-app-3.0.6.jar" && \
    curl -L -o /tmp/jai-imageio-core.jar \
    "https://repo1.maven.org/maven2/com/github/jai-imageio/jai-imageio-core/1.4.0/jai-imageio-core-1.4.0.jar" && \
    curl -L -o /tmp/jai-imageio-jpeg2000.jar \
    "https://repo1.maven.org/maven2/com/github/jai-imageio/jai-imageio-jpeg2000/1.4.0/jai-imageio-jpeg2000-1.4.0.jar"

FROM ubuntu:24.04 AS lagom-build

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    curl \
    git \
    ninja-build \
    pkg-config \
    clang \
    libgl1-mesa-dev \
    libpulse-dev \
    libssl-dev \
    ca-certificates \
    unzip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
RUN git clone --depth 1 https://github.com/LaurenzV/serenity.git

WORKDIR /build/serenity/Build/lagom
RUN cmake -GNinja -S /build/serenity/Meta/Lagom -B . \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_C_COMPILER=clang \
    -DCMAKE_CXX_COMPILER=clang++ \
    -DBUILD_LAGOM=ON \
    -DBUILD_SHARED_LIBS=OFF \
    -DENABLE_LAGOM_LIBWEB=OFF \
    -DENABLE_LAGOM_LADYBIRD=OFF
RUN ninja pdf

FROM base AS final

RUN apt-get update && apt-get install -y --no-install-recommends \
    openjdk-17-jre-headless \
    && rm -rf /var/lib/apt/lists/*

COPY --from=node:20-bookworm-slim /usr/local/bin/node /usr/local/bin/node

RUN mkdir -p /opt/bin

COPY --from=pdfium /build/pdfium/target/release/pdfium /opt/bin/pdfium
COPY --from=pdfium /usr/lib/libpdfium.so /usr/lib/libpdfium.so
COPY --from=mupdf-build /build/mupdf-1.26.3-source/build/release/mutool /opt/bin/mutool
COPY --from=ghostscript-build /build/ghostscript-10.06.0/bin/gs /opt/bin/gs
COPY --from=ghostscript-build /build/ghostscript-10.06.0/Resource /opt/ghostscript/Resource
COPY --from=ghostscript-build /build/ghostscript-10.06.0/iccprofiles /opt/ghostscript/iccprofiles
COPY --from=pdfjs /opt/pdfjs /opt/pdfjs
COPY --from=pdfbox-download /tmp/pdfbox.jar /opt/pdfbox/pdfbox.jar
COPY --from=pdfbox-download /tmp/jai-imageio-core.jar /opt/pdfbox/jai-imageio-core.jar
COPY --from=pdfbox-download /tmp/jai-imageio-jpeg2000.jar /opt/pdfbox/jai-imageio-jpeg2000.jar
COPY --from=lagom-build /build/serenity/Build/lagom/bin/pdf /opt/bin/serenity-pdf
COPY --from=lagom-build /build/serenity/Build/lagom/Root/res /opt/serenity/res

RUN ldconfig

COPY docker/entrypoint.sh /opt/bin/entrypoint.sh
RUN chmod +x /opt/bin/entrypoint.sh

ENV GS_LIB=/opt/ghostscript/Resource/Init:/opt/ghostscript/Resource/Font:/opt/ghostscript/iccprofiles
ENV PATH="/opt/bin:${PATH}"

WORKDIR /work
ENTRYPOINT ["/opt/bin/entrypoint.sh"]
