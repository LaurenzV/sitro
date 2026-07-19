FROM ubuntu:26.04 AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libfreetype6 \
    libjpeg62 \
    libpng16-16t64 \
    libtiff6 \
    libopenjp2-7 \
    libfontconfig1 \
    liblcms2-2 \
    poppler-data \
    && rm -rf /var/lib/apt/lists/*

FROM rust:latest AS rust-build

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
RUN PDFIUM_ARCH=$([ "$TARGETARCH" = "arm64" ] && echo "arm64" || echo "x64") && \
    curl -L -o /tmp/pdfium.tgz \
    "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7947/pdfium-linux-${PDFIUM_ARCH}.tgz" && \
    mkdir -p /opt/pdfium && \
    tar -xzf /tmp/pdfium.tgz -C /opt/pdfium && \
    rm /tmp/pdfium.tgz

FROM rust-build AS pdfium

WORKDIR /build/pdfium
COPY src/pdfium/Cargo.toml src/pdfium/Cargo.lock* ./
COPY src/pdfium/src ./src

RUN cp /opt/pdfium/lib/libpdfium.so /usr/lib/ && ldconfig
RUN cargo build --release

FROM base AS poppler-build

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    curl \
    pkg-config \
    libboost-dev \
    libfontconfig1-dev \
    libfreetype6-dev \
    libjpeg-dev \
    liblcms2-dev \
    libopenjp2-7-dev \
    libpng-dev \
    libtiff-dev \
    zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
RUN curl -L -o poppler.tar.xz \
    "https://poppler.freedesktop.org/poppler-26.07.0.tar.xz" \
    && tar -xf poppler.tar.xz \
    && rm poppler.tar.xz

WORKDIR /build/poppler-26.07.0
RUN cmake -S . -B build \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_SHARED_LIBS=OFF \
    -DBUILD_GTK_TESTS=OFF \
    -DBUILD_QT5_TESTS=OFF \
    -DBUILD_QT6_TESTS=OFF \
    -DBUILD_CPP_TESTS=OFF \
    -DBUILD_MANUAL_TESTS=OFF \
    -DENABLE_CPP=OFF \
    -DENABLE_GLIB=OFF \
    -DENABLE_QT5=OFF \
    -DENABLE_QT6=OFF \
    -DENABLE_LIBCURL=OFF \
    -DENABLE_NSS3=OFF \
    -DENABLE_GPGME=OFF \
    && cmake --build build --target pdftoppm -j$(nproc)

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
    "https://mupdf.com/downloads/archive/mupdf-1.28.0-source.tar.gz" \
    && tar -xzf mupdf.tar.gz \
    && rm mupdf.tar.gz

WORKDIR /build/mupdf-1.28.0-source
RUN make HAVE_X11=no HAVE_GLUT=no -j$(nproc)

FROM --platform=$BUILDPLATFORM ubuntu:26.04 AS ghostscript-source

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
RUN curl -L -o gs.tar.gz \
    "https://github.com/ArtifexSoftware/ghostpdl-downloads/releases/download/gs10071/ghostscript-10.07.1.tar.gz" \
    && tar -xzf gs.tar.gz \
    && rm gs.tar.gz

FROM base AS ghostscript-build

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    libfreetype6-dev \
    libjpeg-dev \
    libpng-dev \
    libtiff-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=ghostscript-source /build/ghostscript-10.07.1 /build/ghostscript-10.07.1

WORKDIR /build/ghostscript-10.07.1
RUN ./configure && make -j$(nproc)

FROM node:24-bookworm-slim AS pdfjs

WORKDIR /opt/pdfjs
COPY src/pdfjs/package.json src/pdfjs/package-lock.json* ./
RUN npm ci
COPY src/pdfjs/pdfjs_render.mjs ./

FROM base AS pdfbox-download

RUN apt-get update && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*
RUN curl -L -o /tmp/pdfbox.jar \
    "https://dlcdn.apache.org/pdfbox/3.0.8/pdfbox-app-3.0.8.jar" && \
    curl -L -o /tmp/jai-imageio-core.jar \
    "https://repo1.maven.org/maven2/com/github/jai-imageio/jai-imageio-core/1.4.0/jai-imageio-core-1.4.0.jar" && \
    curl -L -o /tmp/jai-imageio-jpeg2000.jar \
    "https://repo1.maven.org/maven2/com/github/jai-imageio/jai-imageio-jpeg2000/1.4.0/jai-imageio-jpeg2000-1.4.0.jar"

FROM ubuntu:26.04 AS lagom-build

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
RUN git init serenity \
    && cd serenity \
    && git remote add origin https://github.com/SerenityOS/serenity.git \
    && git fetch --depth 1 origin e22c1f2df8bdf7dc3ff6f39460d72aaf0c8bbaa5 \
    && git checkout --detach FETCH_HEAD

COPY docker/serenity-scale.patch /tmp/serenity-scale.patch
RUN cd serenity && git apply /tmp/serenity-scale.patch

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
    default-jre-headless \
    && rm -rf /var/lib/apt/lists/*

COPY --from=node:24-bookworm-slim /usr/local/bin/node /usr/local/bin/node

RUN mkdir -p /opt/bin

COPY --from=pdfium /build/pdfium/target/release/pdfium /opt/bin/pdfium
COPY --from=pdfium /usr/lib/libpdfium.so /usr/lib/libpdfium.so
COPY --from=poppler-build /build/poppler-26.07.0/build/utils/pdftoppm /opt/bin/pdftoppm
COPY --from=mupdf-build /build/mupdf-1.28.0-source/build/release/mutool /opt/bin/mutool
COPY --from=ghostscript-build /build/ghostscript-10.07.1/bin/gs /opt/bin/gs
COPY --from=ghostscript-build /build/ghostscript-10.07.1/Resource /opt/ghostscript/Resource
COPY --from=ghostscript-build /build/ghostscript-10.07.1/iccprofiles /opt/ghostscript/iccprofiles
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
